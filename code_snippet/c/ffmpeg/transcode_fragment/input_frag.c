#include <stdio.h>
#include <stdlib.h>
#include <fcntl.h>
#include <unistd.h>
#include <sys/stat.h>

#include "mp4_hdr.h"
#include "input_frag.h"

static int app_read_header(void *opaque, uint8_t *buf, int required_size)
{
    if(!opaque) {
        return AVERROR(EINVAL) ;
    }
    int hdr_tmp_fd = *(int *)opaque;
    int nread = read(hdr_tmp_fd, buf, required_size);
    if(nread == 0) {
        nread = AVERROR_EOF;
    }
    return nread;
} // end of app_read_header

static int app_read_packet_fragment(void *opaque, uint8_t *buf, int required_size)
{
    struct buffer_data *bd = (struct buffer_data *)opaque;
    int cp_size = FFMIN(required_size, bd->size);
    if (cp_size > 0) { //  copy internal buffer data to buf
        memcpy(buf, bd->ptr, cp_size);
        bd->ptr  += cp_size;
        bd->size -= cp_size;
        av_log(NULL, AV_LOG_DEBUG, "[app_read_packet_fragment] buf avail size:%zu, required size:%d \n",
                 bd->size, required_size);
    } else {
        av_log(NULL, AV_LOG_DEBUG, "[app_read_packet_fragment] end of buffer reached, required size:%d \n",
                 required_size);
        cp_size = AVERROR_EOF;
    }
    // if (bd->size == 0) {
    //     bd->ptr  = bd->ptr_bak;
    //     bd->size = read(bd->fd, bd->ptr, bd->size_bak);
    // }
    return cp_size;
} // end of app_read_packet_fragment


static __attribute__((optimize("O0"))) int _fmtctx_parse_media_header(AVFormatContext *fmt_ctx, int src_fd)
{ // TODO, the code here works only for mp4 container
    // when avformat_open_input() read mp4 input container, the function expects to read
    // following atom sequence in  strict order :
    // `ftyp` --> `moov` --> beginning 8 bytes of `mdat`
    char hdr_tmp_fd_path[] = "./tmp_media_header_XXXXXX";
    int hdr_tmp_fd = mkstemp(&hdr_tmp_fd_path[0]);
    int origin_mdat_pos = serialize_mp4_header(hdr_tmp_fd, src_fd);
    lseek(src_fd, origin_mdat_pos, SEEK_SET);
    lseek(hdr_tmp_fd, 0, SEEK_SET);
    fmt_ctx -> pb ->opaque = (void *)&hdr_tmp_fd;
    int ret = avformat_open_input(&fmt_ctx, NULL, NULL, NULL);
    if(ret >= 0) {
        fmt_ctx->pb ->pos = origin_mdat_pos;
    }
    close(hdr_tmp_fd);
    unlink(&hdr_tmp_fd_path[0]);
    return ret;
} // end of _fmtctx_parse_media_header


static __attribute__((optimize("O0"))) int _parse_input_stream(AVFormatContext *fmt_ctx)
{
    int ret =  0;
    int i = 0, j = 0;
    struct buffer_data *bd = fmt_ctx->pb->opaque;
    // the bytes of media file to load should be fixed number of initial packets for each stream,
    // if the first packet is too large to fit in, then the application may discard processing
    // the video file
    size_t sz = 0;
    for (i = 0; i < fmt_ctx->nb_streams; i++) {
        AVStream *stream = fmt_ctx->streams[i];
        size_t nb_init_pkts = FFMIN(15, stream->nb_index_entries);
        for (j = 0; j < nb_init_pkts; j++) {
             sz += stream-> index_entries[j].size;
        }
    }
    fmt_ctx -> probesize = sz;
    av_log(NULL, AV_LOG_INFO, "Buffer size of user application %lu\n", sz);
    //// uint8_t  srcfile_buffer[sz];
    bd->ptr = av_malloc(sz);
    //// bd->ptr = &srcfile_buffer[0];
    bd->ptr_bak = bd->ptr;
    bd->size_bak = sz;
    memset(bd->ptr, 0, sz);
    bd->size = read(bd->fd, bd->ptr, bd->size_bak);
    // avformat_find_stream_info() expects to read the first few frames which will be
    // used later when fetching packets,
    ret = avformat_find_stream_info(fmt_ctx, NULL);
    if(ret < 0) { goto end; }
    bd->stream_ctx = av_mallocz_array(fmt_ctx->nb_streams, sizeof(StreamContext));
    for (i = 0; i < fmt_ctx->nb_streams; i++) {
        AVCodecContext *codec_ctx;
        AVStream *stream = fmt_ctx->streams[i];
        AVCodec *dec = avcodec_find_decoder(stream->codecpar->codec_id);
        if (!dec) {
            av_log(NULL, AV_LOG_ERROR, "Failed to find decoder for stream #%u\n", i);
            ret = AVERROR_DECODER_NOT_FOUND;
            break;
        }
        codec_ctx = avcodec_alloc_context3(dec);
        if (!codec_ctx) {
            av_log(NULL, AV_LOG_ERROR, "Failed to allocate the decoder context for stream #%u\n", i);
            ret = AVERROR(ENOMEM);
            break;
        }
        ret = avcodec_parameters_to_context(codec_ctx, stream->codecpar);
        if (ret < 0) {
            av_log(NULL, AV_LOG_ERROR, "Failed to copy decoder parameters to input decoder context "
                   "for stream #%u\n", i);
            break;
        }
        if (codec_ctx->codec_type == AVMEDIA_TYPE_VIDEO) {
            codec_ctx->framerate = av_guess_frame_rate(fmt_ctx, stream, NULL);
        }
        if (codec_ctx->codec_type == AVMEDIA_TYPE_VIDEO || codec_ctx->codec_type == AVMEDIA_TYPE_AUDIO)
        {
            ret = avcodec_open2(codec_ctx, dec, NULL);
            if (ret < 0) {
                av_log(NULL, AV_LOG_ERROR, "Failed to open decoder for stream #%u\n", i);
                break;
            }
        }
        bd->stream_ctx[i].dec_ctx = codec_ctx;
        bd->stream_ctx[i].last_recovered_pkt_idx = 0;
    } // end of loop
    if(ret >= 0)
        av_dump_format(fmt_ctx, 0, "some_input_file_path", 0);
end:
    return ret;
} // end of _parse_input_stream

void _app_input_deinit(AVFormatContext *fmt_ctx, struct buffer_data *bd)
{
    if(bd->stream_ctx) {
        int idx = 0;
        for (idx = 0; idx < fmt_ctx->nb_streams; idx++) {
            StreamContext *sctx = &bd->stream_ctx[idx];
            if (sctx->dec_ctx)
                avcodec_free_context(&sctx->dec_ctx);
            if (sctx->enc_ctx)
                avcodec_free_context(&sctx->enc_ctx);
            if (sctx->filter_graph) {
                // the function automatically dealloc all registered filter context
                //  objects (AVFilterContext, e.g. sctx->filt_src_ctx, sctx->filt_sink_ctx)
                avfilter_graph_free(&sctx->filter_graph);
            }
        }
        av_freep(&bd->stream_ctx);
    }
    if(bd->ptr_bak) {
        free(bd->ptr_bak);
        bd->ptr_bak = NULL;
        bd->ptr = NULL;
    }
    _app_avfmt_deinit_common(fmt_ctx, bd);
} // end of _app_input_deinit

AVFormatContext *_app_input_init(char *input_filename, struct buffer_data *bd)
{
#define  AVIO_CTX_BUFFER_SIZE 2048
    uint8_t *avio_ctx_buffer = NULL;
    AVIOContext *avio_ctx = NULL;
    AVFormatContext *fmt_ctx = NULL;
    int ret = 0;
    bd->fd =  open(input_filename, O_RDONLY);
    if (bd->fd < 3)
        goto error;
    if (!(fmt_ctx = avformat_alloc_context())) {
        goto error;
    }
    avio_ctx_buffer = av_malloc(AVIO_CTX_BUFFER_SIZE);
    if (!avio_ctx_buffer) {
        goto error;
    }
    avio_ctx = avio_alloc_context(avio_ctx_buffer, AVIO_CTX_BUFFER_SIZE,
              0, bd, &app_read_header, NULL, NULL); // &app_seek_packet
    // I/O seek function will behave abnormally in libav, figure out why (TODO)
    if (!avio_ctx) {
        goto error;
    }
    avio_ctx -> max_packet_size = 4321;
    fmt_ctx->pb = avio_ctx;
    {
        ret = _fmtctx_parse_media_header(fmt_ctx, bd->fd);
        if(ret < 0) { goto error; }
        avio_ctx ->opaque = (void *)bd;
        avio_ctx ->read_packet = app_read_packet_fragment; // change low-level I/O read function
        ret = _parse_input_stream(fmt_ctx);
        if(ret < 0) { goto error; }
    }
    return fmt_ctx;
error:
    if(ret < 0) {
        char errbuf[128];
        av_strerror(ret, &errbuf[0], 128);
        fprintf(stdout, "_app_input_init(), err = (%d)%s \n", ret, &errbuf[0]);
    }
    _app_input_deinit(fmt_ctx, bd);
    return NULL;
#undef  AVIO_CTX_BUFFER_SIZE
} // end of _app_input_init

