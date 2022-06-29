#include <stdlib.h>
#include <unistd.h>
#include <string.h>
#include <sys/stat.h>
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

static __attribute__((optimize("O0"))) void _app_config_dst_encoder(
        AVCodecContext *enc_ctx, AVCodecContext *dec_ctx)
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

int setup_output_stream_codec(AVFormatContext *fmt_o_ctx, AVFormatContext *fmt_i_ctx)
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
        } else { // if this stream must be remuxed
            ret = avcodec_parameters_copy(out_stream->codecpar, in_stream->codecpar);
            if (ret < 0) {
                av_log(NULL, AV_LOG_ERROR, "Copying parameters for stream #%u failed\n", idx);
                break;
            }
            out_stream->time_base = in_stream->time_base;
        }
    } // end of loop
    return ret;
} // end of setup_output_stream_codec


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

int mkdir_recursive(const char *path) {
    int err = 0; 
    size_t path_sz = strlen(path) + 1;
    char *path_dup = strdup(path);
    char  path_parent[path_sz];
    size_t  num_dirs_created = 0;
    char *saveptr = NULL;
    char *tok = NULL;
    memset(&path_parent[0], 0x0, path_sz);
    if(path[0] == '/') {
        path_parent[0] = path[0];
    }
    for(tok = strtok_r(path_dup, "/", &saveptr); tok; tok = strtok_r(NULL, "/", &saveptr))
    {
        if(num_dirs_created > 0) { // not NULL-terminating char
            strncat(&path_parent[0], "/", 1);
        }
        strncat(&path_parent[0], tok, strlen(tok));
        if (access(&path_parent[0], F_OK) == 0) {
            // skip, folder already created
        } else if(mkdir(&path_parent[0], S_IRWXU) != 0) {
            err = 1;
            av_log(NULL, AV_LOG_ERROR, "Failed to create directory : %s \n", path);
            break;
        }
        num_dirs_created++;
    }
    free(path_dup);
    return err;
} // end of mkdir_recursive

