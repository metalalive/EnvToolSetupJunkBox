#include <libavutil/avutil.h>
#include <libavutil/rational.h>

#include "common.h"

void _app_avfmt_deinit_common(AVFormatContext *fmt_ctx, struct buffer_data *bd)
{
    if(fmt_ctx) {
        AVIOContext *avio_ctx = fmt_ctx->pb;
        /* note: the internal buffer could have changed, and be != avio_ctx_buffer */
        if (avio_ctx)
            av_freep(&avio_ctx->buffer);
        avio_context_free(&fmt_ctx->pb);
        avformat_close_input(&fmt_ctx); // will automatically invoke avformat_free_context()
    }
    if(bd->fd >= 3) {
        close(bd->fd);
        bd->fd = -1;
    }
}

void _app_config_dst_encoder(AVCodecContext *enc_ctx, AVCodecContext *dec_ctx)
{
    // In this example, we transcode to same properties (picture size,
    // sample rate etc.). These properties can be changed for output
    // streams easily using filters
    const AVCodec *encoder = enc_ctx->codec;
    if (dec_ctx->codec_type == AVMEDIA_TYPE_VIDEO) {
        enc_ctx->height = ((uint32_t) (dec_ctx->height * 2/ 3)) & 0xfffffffe; // downsize width/height
        enc_ctx->width  = ((uint32_t) (dec_ctx->width  * 2/ 3)) & 0xfffffffe;
        enc_ctx->sample_aspect_ratio = dec_ctx->sample_aspect_ratio;
        // take first format from list of supported formats
        if (encoder->pix_fmts)
            enc_ctx->pix_fmt = encoder->pix_fmts[0];
        else
            enc_ctx->pix_fmt = dec_ctx->pix_fmt;
        // video time_base can be set to whatever is handy and supported by encoder
        ////enc_ctx->time_base = av_inv_q(dec_ctx->framerate);
        enc_ctx->time_base = dec_ctx->time_base;
        enc_ctx->framerate = (AVRational){num:11, den:1}; // dec_ctx->framerate;
    } else { // AVMEDIA_TYPE_AUDIO
        enc_ctx->sample_rate = 44100; // dec_ctx->sample_rate; , change sample rate will make it sound `digital`
        enc_ctx->bit_rate    = 63999; // dec_ctx->bit_rate; , will reduce file size
        enc_ctx->channel_layout = dec_ctx->channel_layout;
        enc_ctx->channels = av_get_channel_layout_nb_channels(enc_ctx->channel_layout);
        // take first format from list of supported formats
        enc_ctx->sample_fmt = encoder->sample_fmts[0];
        enc_ctx->time_base = (AVRational){1, enc_ctx->sample_rate};
    }
} // end of _app_config_dst_encoder

int  _av_stream_index_lookup(AVStream *in_stream, size_t target_f_pos, size_t start_idx)
{
    int sample_idx = -1;
    for(size_t idx = start_idx; idx < in_stream->nb_index_entries; idx++) {
        AVIndexEntry *entry = &in_stream->index_entries[ idx ];
        if(entry ->pos == target_f_pos) {
            sample_idx = idx;
            break;
        }
    }
    return sample_idx; 
} // end of _av_stream_index_lookup

