#include <assert.h>
#include <stdio.h>
#include <stdlib.h>
#include <fcntl.h>
#include <unistd.h>
#include <sys/stat.h>
#include <string.h>

#include <libavformat/avformat.h>
#include <libavutil/file.h>

#include "output_file.h"

static int64_t app_seek_packet(void *opaque, int64_t offset, int whence)
{
    av_log(NULL, AV_LOG_DEBUG, "[app_seek_packet] offset:%ld, whence:%d \n",
            offset, whence );
    struct buffer_data *bd = (struct buffer_data *)opaque;
    return  lseek(bd->fd, offset, whence);
} // end of app_seek_packet

static int app_write_packet(void *opaque, uint8_t *buf, int buf_size)
{
    av_log(NULL, AV_LOG_DEBUG, "[app_write_packet] buf:%p, size:%d \n", buf, buf_size);
    struct buffer_data *bd = (struct buffer_data *)opaque;
    write(bd->fd, buf, buf_size);
    return buf_size;
}

static __attribute__((optimize("O0"))) int _parse_output_stream(AVFormatContext *fmt_o_ctx, AVFormatContext *fmt_i_ctx)
{
    int ret = 0;
    int idx = 0;
    struct buffer_data *bd = fmt_i_ctx->pb->opaque;
    for (idx = 0; idx < fmt_i_ctx->nb_streams; idx++) {
        AVStream *in_stream = fmt_i_ctx->streams[idx];
        AVStream *out_stream = avformat_new_stream(fmt_o_ctx, NULL);
        if (!out_stream) {
            av_log(NULL, AV_LOG_ERROR, "Failed allocating output stream\n");
            ret = AVERROR_UNKNOWN;
            break;
        }
        AVCodecContext *dec_ctx = bd->stream_ctx[idx].dec_ctx;
        if(dec_ctx->codec_type == AVMEDIA_TYPE_VIDEO || dec_ctx->codec_type == AVMEDIA_TYPE_AUDIO)
        {
            AVCodec *encoder = avcodec_find_encoder(dec_ctx->codec_id);
            if (!encoder) {
                av_log(NULL, AV_LOG_FATAL, "Necessary encoder not found\n");
                ret = AVERROR_INVALIDDATA;
                break;
            }
            AVCodecContext *enc_ctx = avcodec_alloc_context3(encoder);
            if (!enc_ctx) {
                av_log(NULL, AV_LOG_FATAL, "Failed to allocate the encoder context\n");
                ret = AVERROR(ENOMEM);
                break;
            }
            _app_config_dst_encoder(enc_ctx, dec_ctx);
            if (fmt_o_ctx->oformat->flags & AVFMT_GLOBALHEADER) {
                enc_ctx->flags |= AV_CODEC_FLAG_GLOBAL_HEADER;
            }
            // Third parameter can be used to pass settings to encoder
            ret = avcodec_open2(enc_ctx, encoder, NULL);
            if (ret < 0) {
                av_log(NULL, AV_LOG_ERROR, "Cannot open video encoder for stream #%u\n", idx);
                break;
            }
            ret = avcodec_parameters_from_context(out_stream->codecpar, enc_ctx);
            if (ret < 0) {
                av_log(NULL, AV_LOG_ERROR, "Failed to copy encoder parameters to output stream #%u\n", idx);
                break;
            }
            out_stream->time_base = enc_ctx->time_base;
            bd->stream_ctx[idx].enc_ctx = enc_ctx;
        } else if (dec_ctx->codec_type == AVMEDIA_TYPE_UNKNOWN) {
            av_log(NULL, AV_LOG_FATAL, "Elementary stream #%d is of unknown type, cannot proceed\n", idx);
            ret = AVERROR_INVALIDDATA;
            break;
        } else {
            // if this stream must be remuxed
            ret = avcodec_parameters_copy(out_stream->codecpar, in_stream->codecpar);
            if (ret < 0) {
                av_log(NULL, AV_LOG_ERROR, "Copying parameters for stream #%u failed\n", idx);
                break;
            }
            out_stream->time_base = in_stream->time_base;
        }
    } // end of loop
    if(!ret) {
        assert(fmt_o_ctx->oformat->flags & AVFMT_NOFILE);
        // init muxer, write output file header
        AVDictionary *options = NULL;
        // For non-streaming output, the option `frag_size` causes data corruption in output file
        //// ret = av_dict_set_int(&options, "frag_size", (int64_t)fmt_o_ctx->pb->buffer_size, 0);
        ret = avformat_write_header(fmt_o_ctx, &options);
        if (ret < 0) {
            char errbuf[128];
            av_strerror(ret, &errbuf[0], 128);
            av_log(NULL, AV_LOG_ERROR, "Error occurred when opening output file, %s \n", &errbuf[0]);
        }
        av_dict_free(&options);
        int is_output = 1;
        av_dump_format(fmt_o_ctx, 0, "some_output_file_path", is_output);
    }
    return ret;
} // end of _parse_output_stream


void _app_output_deinit(AVFormatContext *fmt_ctx, struct buffer_data *bd)
{
    bd->stream_ctx = NULL;
    if(fmt_ctx)
        _app_avfmt_deinit_common(fmt_ctx, bd);
} // end of _app_output_deinit


AVFormatContext *_app_output_init(char *filename, struct buffer_data *bd, AVFormatContext *fmt_in_ctx)
{
    assert(fmt_in_ctx && fmt_in_ctx->iformat);
#define  AVIO_CTX_BUFFER_SIZE 4096
    uint8_t *avio_ctx_buffer = NULL;
    AVIOContext *avio_ctx = NULL;
    AVFormatContext *fmt_ctx = NULL;
    int ret = 0;
    bd->fd =  open(filename, O_CREAT | O_RDWR);
    if (bd->fd < 3)
        goto error;
    if (fchmod(bd->fd, S_IRUSR | S_IWUSR) != 0)
        goto error;
    {
        // strtok will return junk data if first argument `str` is a local array (not allocated dynamically)
        // figure out why does it happen (TODO)
        //// char fmt_in_hint[128] = {0};
        //// memcpy(&fmt_in_hint[0], fmt_in_ctx->iformat->name, strlen(fmt_in_ctx->iformat->name));
        char *fmt_in_hint = strdup(fmt_in_ctx->iformat->name);
        char *saveptr = NULL;
        char *tok = strtok_r(&fmt_in_hint[0], ",", &saveptr);
        ret = avformat_alloc_output_context2(&fmt_ctx, NULL, tok, NULL);
        free(fmt_in_hint);
        // ret = avformat_alloc_output_context2(&fmt_ctx, NULL, "mp4", NULL);
        if (ret < 0 || !fmt_ctx) {
            goto error;
        }
    }
    avio_ctx_buffer = av_malloc(AVIO_CTX_BUFFER_SIZE);
    if (!avio_ctx_buffer) { goto error; }
    // For non-streaming output, seek function is ESSENTIAL at the end of transcoding
    // process, to update different fields of `mdat` and `moov` atom
    int write_flag = 1;
    avio_ctx = avio_alloc_context(avio_ctx_buffer, AVIO_CTX_BUFFER_SIZE, write_flag,
            bd, NULL, &app_write_packet, &app_seek_packet);
    if (!avio_ctx) { goto error; }
    fmt_ctx->oformat->flags |=  AVFMT_NOFILE;
    fmt_ctx->pb = avio_ctx;
    ret = _parse_output_stream(fmt_ctx, fmt_in_ctx);
    if(ret < 0) { goto error; }
    { // Output Format Context shares the same stream context with Input Formaat Context
        struct buffer_data *bd_i = fmt_in_ctx->pb->opaque;
        struct buffer_data *bd_o = fmt_ctx->pb->opaque;
        assert(bd == bd_o);
        bd_o->stream_ctx = bd_i->stream_ctx;
    }
    return fmt_ctx;
error:
    _app_output_deinit(fmt_ctx, bd);
    return NULL;
#undef  AVIO_CTX_BUFFER_SIZE
} // end of _app_output_init
