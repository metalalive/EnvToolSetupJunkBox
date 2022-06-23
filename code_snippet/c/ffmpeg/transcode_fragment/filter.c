#include <libavformat/avformat.h>
#include <libavformat/avio.h>
#include <libavfilter/buffersink.h>
#include <libavfilter/buffersrc.h>
#include <libavutil/opt.h>
#include <libavutil/pixdesc.h>

#include "common.h"
#include "filter.h"

static __attribute__((optimize("O0"))) int init_filter_video(
        StreamContext *sctx, AVFilterGraph *filt_graph, AVFilterContext **filt_ctx_src, AVFilterContext **filt_ctx_sink)
{
    int ret = 0;
    char args[512] = {0};
    const AVFilter *buffersrc  = avfilter_get_by_name("buffer");
    const AVFilter *buffersink = avfilter_get_by_name("buffersink");
    if (!buffersrc || !buffersink) {
        av_log(NULL, AV_LOG_ERROR, "filtering source or sink element not found\n");
        ret = AVERROR_UNKNOWN;
        goto end;
    }
    AVCodecContext *dec_ctx = sctx->dec_ctx;
    AVCodecContext *enc_ctx = sctx->enc_ctx;
    snprintf(args, sizeof(args),
            "video_size=%dx%d:pix_fmt=%d:time_base=%d/%d:pixel_aspect=%d/%d",
            dec_ctx->width, dec_ctx->height, dec_ctx->pix_fmt,
            dec_ctx->time_base.num, dec_ctx->time_base.den,
            dec_ctx->sample_aspect_ratio.num,
            dec_ctx->sample_aspect_ratio.den );
    ret = avfilter_graph_create_filter(filt_ctx_src, buffersrc, "in", args, NULL, filt_graph);
    if (ret < 0) {
        av_log(NULL, AV_LOG_ERROR, "[Filter] Failed to create buffer source\n");
        goto end;
    }
    ret = avfilter_graph_create_filter(filt_ctx_sink, buffersink, "out",  NULL, NULL, filt_graph);
    if (ret < 0) {
        av_log(NULL, AV_LOG_ERROR, "[Filter] Failed to create buffer sink\n");
        goto end;
    }
    ret = av_opt_set_bin(*filt_ctx_sink, "pix_fmts", (uint8_t*)&enc_ctx->pix_fmt,
            sizeof(enc_ctx->pix_fmt), AV_OPT_SEARCH_CHILDREN);
    if (ret < 0) {
        av_log(NULL, AV_LOG_ERROR, "Cannot set output pixel format\n");
    }
end:
    return ret;
} // end of init_filter_video


static __attribute__((optimize("O0"))) int init_filter_audio(
        StreamContext *sctx, AVFilterGraph *filt_graph, AVFilterContext **filt_ctx_src, AVFilterContext **filt_ctx_sink)
{
    int ret = 0;
    char args[512] = {0};
    const AVFilter *buffersrc  = avfilter_get_by_name("abuffer");
    const AVFilter *buffersink = avfilter_get_by_name("abuffersink");
    if(!buffersrc || !buffersink) {
        av_log(NULL, AV_LOG_ERROR, "filtering source or sink element not found\n");
        ret = AVERROR_UNKNOWN;
        goto end;
    }
    AVCodecContext *dec_ctx = sctx->dec_ctx;
    AVCodecContext *enc_ctx = sctx->enc_ctx;
    if (!dec_ctx->channel_layout)
        dec_ctx->channel_layout = av_get_default_channel_layout(dec_ctx->channels);
    snprintf(args, sizeof(args),
            "time_base=%d/%d:sample_rate=%d:sample_fmt=%s:channel_layout=0x%"PRIx64,
            dec_ctx->time_base.num, dec_ctx->time_base.den, dec_ctx->sample_rate,
            av_get_sample_fmt_name(dec_ctx->sample_fmt),
            dec_ctx->channel_layout);
    ret = avfilter_graph_create_filter(filt_ctx_src, buffersrc, "in", args, NULL, filt_graph);
    if (ret < 0) {
        av_log(NULL, AV_LOG_ERROR, "Cannot create audio buffer source\n");
        goto end;
    }
    ret = avfilter_graph_create_filter(filt_ctx_sink, buffersink, "out", NULL, NULL, filt_graph);
    if (ret < 0) {
        av_log(NULL, AV_LOG_ERROR, "Cannot create audio buffer sink\n");
        goto end;
    }
    ret = av_opt_set_bin(*filt_ctx_sink, "sample_fmts", (uint8_t*)&enc_ctx->sample_fmt,
            sizeof(enc_ctx->sample_fmt),  AV_OPT_SEARCH_CHILDREN);
    if (ret < 0) {
        av_log(NULL, AV_LOG_ERROR, "Cannot set output sample format\n");
        goto end;
    }
    ret = av_opt_set_bin(*filt_ctx_sink, "channel_layouts", (uint8_t*)&enc_ctx->channel_layout,
            sizeof(enc_ctx->channel_layout), AV_OPT_SEARCH_CHILDREN);
    if (ret < 0) {
        av_log(NULL, AV_LOG_ERROR, "Cannot set output channel layout\n");
        goto end;
    }
    const int out_sample_rates[] = { enc_ctx->sample_rate, -1 };
    ret = av_opt_set_int_list(*filt_ctx_sink, "sample_rates", out_sample_rates, -1,
                              AV_OPT_SEARCH_CHILDREN);
    if (ret < 0) {
        av_log(NULL, AV_LOG_ERROR, "Cannot set output sample rate\n");
    }
end:
    return ret;
} // end of init_filter_audio


int init_filters(AVFormatContext *fmt_i_ctx)
{
    int ret = 0;
    int idx = 0;
    int (*init_filter_fns[2])(StreamContext *, AVFilterGraph *, AVFilterContext **,
            AVFilterContext **) = {init_filter_video, init_filter_audio};
    struct buffer_data *bd = fmt_i_ctx->pb->opaque;
    for (idx = 0; (!ret) && (idx < fmt_i_ctx->nb_streams); idx++)
    {
        StreamContext *sctx = &bd->stream_ctx[idx];
        sctx->filt_sink_ctx = NULL;
        sctx->filt_src_ctx = NULL;
        sctx->filter_graph = NULL;
        char filter_spec[128] = {0};
        enum AVMediaType codectype = fmt_i_ctx->streams[idx]->codecpar->codec_type;
        AVRational frm_ratio = av_mul_q(sctx->dec_ctx->framerate, sctx->dec_ctx->time_base);
        frm_ratio = av_inv_q(frm_ratio) ;
        switch (codectype) {
            case AVMEDIA_TYPE_VIDEO: // passthrough filter specification for video
                // NOTE, `setpts` MUST NOT be present after `fps`, otherwise `fps` stop working silently
                snprintf(filter_spec, sizeof(filter_spec), "fps=%d,setpts=PTS*%f,scale=%d:%d",
                    sctx->enc_ctx->framerate.num,
                    (((float)frm_ratio.num / frm_ratio.den) * ((float)sctx->dec_ctx->framerate.num / sctx->enc_ctx->framerate.num)),
                    //// frm_ratio.num, frm_ratio.den, sctx->dec_ctx->framerate.num, sctx->enc_ctx->framerate.num,
                    sctx->enc_ctx->width, sctx->enc_ctx->height
                );
                break;  // or pass "null" as default filter spec
            case AVMEDIA_TYPE_AUDIO: // passthrough filter specification for audio
                snprintf(filter_spec, sizeof(filter_spec), "aresample=%d", sctx->enc_ctx->sample_rate);
                break;  // or pass "anull" as default filter spec
            default:
                continue;
        }
        AVFilterContext *buffersrc_ctx = NULL;
        AVFilterContext *buffersink_ctx = NULL;
        AVFilterInOut *outputs = avfilter_inout_alloc();
        AVFilterInOut *inputs  = avfilter_inout_alloc();
        AVFilterGraph *filter_graph = avfilter_graph_alloc();
        if (!outputs || !inputs || !filter_graph) {
            ret = AVERROR(ENOMEM);
            goto end;
        }

        ret = init_filter_fns[codectype](sctx, filter_graph, &buffersrc_ctx, &buffersink_ctx);
        if(ret) { goto end; }
        /* Endpoints for the filter graph. */
        outputs->name       = av_strdup("in");
        outputs->filter_ctx = buffersrc_ctx;
        outputs->pad_idx    = 0;
        outputs->next       = NULL;

        inputs->name       = av_strdup("out");
        inputs->filter_ctx = buffersink_ctx;
        inputs->pad_idx    = 0;
        inputs->next       = NULL;

        if (!outputs->name || !inputs->name) {
            ret = AVERROR(ENOMEM);
            goto end;
        }
        ret = avfilter_graph_parse_ptr(filter_graph, &filter_spec[0], &inputs, &outputs, NULL);
        if (ret < 0) { goto end; }
        ret = avfilter_graph_config(filter_graph, NULL);
        if (ret < 0) { goto end; }
        /* Fill FilteringContext */
        sctx->filt_src_ctx  = buffersrc_ctx;
        sctx->filt_sink_ctx = buffersink_ctx;
        sctx->filter_graph  = filter_graph;
end:
        avfilter_inout_free(&inputs);
        avfilter_inout_free(&outputs);
    } // end of loop
    return ret;
} // end of init_filters

