#ifndef APP_COMMON_H
#define APP_COMMON_H
#ifdef __cplusplus
extern "C" {
#endif

#include <libavformat/avformat.h>
#include <libavcodec/avcodec.h>
#include <libavfilter/avfilter.h>
#include <libavutil/error.h>

typedef struct {
    AVCodecContext *dec_ctx;
    AVCodecContext *enc_ctx;
    AVFilterContext *filt_sink_ctx;
    AVFilterContext *filt_src_ctx;
    AVFilterGraph   *filter_graph;
    size_t last_recovered_pkt_idx;
} StreamContext;

struct buffer_data {
    uint8_t *ptr;
    uint8_t *ptr_bak;
    size_t size; ///< size left in the buffer
    size_t size_bak;
    int fd;
    void (*flush_output)(struct buffer_data *);
    int  (*refill_input)(AVFormatContext *, AVPacket *);
    StreamContext *stream_ctx;
    void *priv_data;
};

void _app_avfmt_deinit_common(AVFormatContext *fmt_ctx, struct buffer_data *bd);

int setup_output_stream_codec(AVFormatContext *fmt_o_ctx, AVFormatContext *fmt_i_ctx);

int  _av_stream_index_lookup(AVStream *in_stream, size_t target_f_pos, size_t start_idx);

int mkdir_recursive(const char *path);

#ifdef __cplusplus
} // end of extern C clause
#endif
#endif // end of  APP_COMMON_H
