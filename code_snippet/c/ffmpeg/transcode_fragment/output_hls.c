#include <assert.h>
#include <unistd.h>
#include <dirent.h>
#include <sys/stat.h>
#include <libavformat/avio.h>

#include "common.h"
#include "output_hls.h"

#define  AVIO_CTX_BUFFER_SIZE 4096
#define  TMP_SEGMENTS_BASEPATH  "tmp_hls_seg/version0/"
#define  SEGMENT_FILENAME_PREFIX     "data_seg_"
#define  SEGMENT_FILENAME_TEMPLATE   SEGMENT_FILENAME_PREFIX "%03d"
#define  PLAYLIST_FILENAME   "mystream.m3u8"
#define  FMP4_FILENAME       "init_packet_map"

typedef struct {
    char  *dst_filepath;
    int    processing_seg_num;
} hls_ctx_t;

static __attribute__((optimize("O0"))) void hls_file_to_dst(
        const char *src_filename, const char *src_dir, const char *dst_dir)
{
    size_t src_fullpath_sz = strlen(src_dir) + strlen(src_filename) + 2;
    size_t dst_fullpath_sz = strlen(dst_dir) + strlen(src_filename) + 2;
    char src_fullpath[src_fullpath_sz];
    char dst_fullpath[dst_fullpath_sz];
    memset(&src_fullpath[0], 0, src_fullpath_sz);
    memset(&dst_fullpath[0], 0, dst_fullpath_sz);
    snprintf(&src_fullpath[0], src_fullpath_sz, "%s/%s", src_dir, src_filename);
    snprintf(&dst_fullpath[0], dst_fullpath_sz, "%s/%s", dst_dir, src_filename);
    rename(&src_fullpath[0], &dst_fullpath[0]);
} // end of hls_file_to_dst


static __attribute__((optimize("O0"))) int scan_filter__tmp_hls_file(const struct dirent *d)
{ // 
    int not_eq = strncmp(d->d_name, SEGMENT_FILENAME_PREFIX, strlen(SEGMENT_FILENAME_PREFIX));
    return not_eq == 0;
} // end of scan_filter__tmp_hls_file

static void _hls_file_flush_output(struct buffer_data *bd)
{
    hls_ctx_t *hlsctx = (hls_ctx_t *)bd->priv_data;
    struct dirent **files_stat = NULL;
    int idx = 0;
    int num_files = scandir(TMP_SEGMENTS_BASEPATH, &files_stat, scan_filter__tmp_hls_file, NULL);
    if(!files_stat || num_files <= 0) {
        goto done;
    } // report error
    const char *playlist_path = TMP_SEGMENTS_BASEPATH  PLAYLIST_FILENAME;
    uint8_t playlist_exist = access(playlist_path, F_OK) == 0;
    int curr_max_seg_num = 0;
    size_t seg_prefix_sz = strlen(SEGMENT_FILENAME_PREFIX);
    for(idx = 0; idx < num_files; idx++) {
        char *filename = files_stat[idx]->d_name;
        int _seg_num = strtol(&filename[seg_prefix_sz], NULL, 10);
        curr_max_seg_num = FFMAX(curr_max_seg_num, _seg_num);
    }
    if(playlist_exist) {
        hls_file_to_dst(FMP4_FILENAME,     TMP_SEGMENTS_BASEPATH, hlsctx->dst_filepath);
        hls_file_to_dst(PLAYLIST_FILENAME, TMP_SEGMENTS_BASEPATH, hlsctx->dst_filepath);
        curr_max_seg_num++; // at this moment, the final segment should be ready to  move
    } // check whether playlist is generated, which indicates the end of transcoding process
    if(curr_max_seg_num > hlsctx->processing_seg_num) {
        // move the segment file when you are sure that libavformat completes writing all the necessary bytes to it
        const char *seg_filename_template =  SEGMENT_FILENAME_TEMPLATE;
        size_t src_filename_sz = strlen(seg_filename_template) + 3 + 1;
        char src_filename[src_filename_sz];
        for(idx = hlsctx->processing_seg_num; idx < curr_max_seg_num; idx++) {
            memset(&src_filename[0], 0, src_filename_sz);
            snprintf(&src_filename[0], src_filename_sz, seg_filename_template, idx);
            hls_file_to_dst(&src_filename[0], TMP_SEGMENTS_BASEPATH , hlsctx->dst_filepath);
        }
        hlsctx->processing_seg_num = curr_max_seg_num;
    }
done:
    if(files_stat) {
        for(idx = 0; idx < num_files; idx++) {
            free(files_stat[idx]);
        }
        free(files_stat);
    }
} // end of _hls_file_flush_output


static __attribute__((optimize("O0"))) int _parse_output_stream(AVFormatContext *fmt_o_ctx, AVFormatContext *fmt_i_ctx)
{
    int ret = setup_output_stream_codec(fmt_o_ctx, fmt_i_ctx);
    if(!ret) {
        assert(fmt_o_ctx->oformat->flags & AVFMT_NOFILE);
        AVDictionary *options = NULL;
        av_dict_set_int(&options, "hls_playlist_type", (int64_t)2, 0); // vod
        av_dict_set_int(&options, "hls_segment_type", (int64_t)1, 0); // fmp4
        av_dict_set_int(&options, "hls_time", (int64_t)7, 0);
        av_dict_set_int(&options, "hls_delete_threshold", (int64_t)2, 0);
        // 1000 KB, not implemented yet in ffmpeg
        //// av_dict_set_int(&options, "hls_segment_size", (int64_t)1024000, 0);
        // will be prepended to each segment entry in final playlist
        //// av_dict_set(&options, "hls_base_url", "/file?id=x4eyy5i&segment=", 0);
        av_dict_set(&options, "hls_segment_filename",   TMP_SEGMENTS_BASEPATH SEGMENT_FILENAME_TEMPLATE, 0);
        av_dict_set(&options, "hls_fmp4_init_filename", FMP4_FILENAME, 0);
        // avformat_write_header() will NOT write any bytes to playlist, is it normal behaviour ?
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
    if(bd->priv_data) {
        hls_ctx_t *hlsctx = (hls_ctx_t *)bd->priv_data;
        av_freep(&hlsctx->dst_filepath);
        av_freep(&bd->priv_data);
    }
    if(fmt_ctx)
        _app_avfmt_deinit_common(fmt_ctx, bd);
} // end of _app_output_deinit


AVFormatContext *_app_output_init(char *dst_path, struct buffer_data *bd, AVFormatContext *fmt_in_ctx)
{
    assert(fmt_in_ctx && fmt_in_ctx->iformat);
    uint8_t *avio_ctx_buffer = NULL;
    AVIOContext *avio_ctx = NULL;
    AVFormatContext *fmt_ctx = NULL;
    int ret = 0;
    bd->fd = -1;
    if(mkdir_recursive(TMP_SEGMENTS_BASEPATH) != 0) {
        goto error;
    } // create folder as temporary local storage for generated segment files
    {
        struct stat file_stat;
        if(stat(dst_path, &file_stat) == 0) {
            if(!S_ISDIR(file_stat.st_mode)) {
                av_log(NULL, AV_LOG_ERROR, "existing path is not directory : %s \n", dst_path);
                goto error;
            }
        } else if(mkdir_recursive(dst_path) != 0) {
            goto error;
        }
    }
    ret = avformat_alloc_output_context2(&fmt_ctx, NULL, "hls", TMP_SEGMENTS_BASEPATH  PLAYLIST_FILENAME);
    if (ret < 0 || !fmt_ctx) {
        goto error;
    }
    avio_ctx_buffer = av_malloc(AVIO_CTX_BUFFER_SIZE);
    if (!avio_ctx_buffer) { goto error; }
    int write_flag = 1;
    avio_ctx = avio_alloc_context(avio_ctx_buffer, AVIO_CTX_BUFFER_SIZE,
            write_flag, bd, NULL, NULL, NULL);
    if (!avio_ctx) { goto error; }
    fmt_ctx->pb = avio_ctx;
    ret = _parse_output_stream(fmt_ctx, fmt_in_ctx);
    if (ret < 0) { goto error; }
    { // Output Format Context shares the same stream context with Input Formaat Context
        struct buffer_data *bd_i = fmt_in_ctx->pb->opaque;
        struct buffer_data *bd_o = fmt_ctx->pb->opaque;
        assert(bd == bd_o);
        bd_o->stream_ctx = bd_i->stream_ctx;
    }
    {
        hls_ctx_t *hlsctx = av_malloc(sizeof(hls_ctx_t));
        hlsctx->dst_filepath = strdup(dst_path);
        hlsctx->processing_seg_num = 0;
        bd->priv_data = (void *)hlsctx;
        bd->flush_output = _hls_file_flush_output;
    }
    return fmt_ctx;
error:
    _app_output_deinit(fmt_ctx, bd);
    return NULL;
} // end of _app_output_init

