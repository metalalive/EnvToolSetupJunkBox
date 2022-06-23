#include <assert.h>
#include <libavfilter/buffersink.h>
#include <libavfilter/buffersrc.h>

#include "common.h"
#include "packet.h"

static __attribute__((optimize("O0"))) int encode_write_frame_v2(
        AVFormatContext *fmt_o_ctx, AVFrame *filt_frame, int stream_idx, int *got_frame)
{
    int ret;
    int got_frame_local = 0;
    AVPacket enc_pkt = {.data = NULL, .size = 0};
    struct buffer_data *bd = fmt_o_ctx->pb->opaque;
    StreamContext *sctx = &bd->stream_ctx[stream_idx];
    av_init_packet(&enc_pkt);
    if (!got_frame)
        got_frame = &got_frame_local;
    ret = avcodec_send_frame(sctx->enc_ctx, filt_frame);
    if (ret >= 0) {
        av_log(NULL, AV_LOG_DEBUG, "Muxing frame\n");
        *got_frame = 1;
    } else {
        av_log(NULL, AV_LOG_ERROR, "Error sending a frame for encoding\n");
    }
    while (ret >= 0 && *got_frame) {
        ret = avcodec_receive_packet(sctx->enc_ctx, &enc_pkt);
        if (ret == AVERROR(EAGAIN) || ret == AVERROR_EOF) {
            *got_frame = 0;
            ret = 0;
        } else if (ret >= 0) { // prepare packet for muxing
            enc_pkt.stream_index = stream_idx;
            av_packet_rescale_ts(&enc_pkt, sctx->enc_ctx->time_base, fmt_o_ctx->streams[stream_idx]->time_base);
            av_log(NULL, AV_LOG_DEBUG, "Write packet %3"PRId64" (size=%5d)\n", enc_pkt.pts, enc_pkt.size);
            // mux encoded frame, TODO, async write with event loop support
            ret = av_interleaved_write_frame(fmt_o_ctx, &enc_pkt);
        } else if (ret < 0) {
            av_log(NULL, AV_LOG_ERROR, "Error during encoding\n");
        }
    }
    av_packet_unref(&enc_pkt);
    return ret;
} // end of encode_write_frame_v2

#if 0
static int encode_write_frame_deprecated(enum AVMediaType stype, AVFormatContext *fmt_o_ctx,
        AVFrame *filt_frame, int stream_idx, int *got_frame)
{
    int ret;
    int got_frame_local = 0;
    AVPacket enc_pkt = {.data = NULL, .size = 0};
    struct buffer_data *bd = fmt_o_ctx->pb->opaque;
    StreamContext *sctx = &bd->stream_ctx[stream_idx];
    if (!got_frame)
        got_frame = &got_frame_local;
    av_log(NULL, AV_LOG_DEBUG, "Encoding frame\n");
    // encode filtered frame
    av_init_packet(&enc_pkt);
    int (*enc_func)(AVCodecContext *, AVPacket *, const AVFrame *, int *);
    enc_func = (stype == AVMEDIA_TYPE_VIDEO) ? avcodec_encode_video2: avcodec_encode_audio2;
    ret = enc_func(sctx->enc_ctx, &enc_pkt, filt_frame, got_frame);
    if (ret < 0) { goto end; }
    if (!(*got_frame)) { goto end; } // no frame to write, return without error
    // prepare packet for muxing
    enc_pkt.stream_index = stream_idx;
    av_packet_rescale_ts(&enc_pkt, sctx->enc_ctx->time_base, fmt_o_ctx->streams[stream_idx]->time_base);
    av_log(NULL, AV_LOG_DEBUG, "Muxing frame\n");
    // mux encoded frame
    ret = av_interleaved_write_frame(fmt_o_ctx, &enc_pkt);
end:
    return ret;
} // end of encode_write_frame_deprecated
#endif

static __attribute__((optimize("O0"))) int flush_encoder( AVFormatContext *fmt_i_ctx,
        AVFormatContext *fmt_o_ctx, int stream_idx)
{
    int ret = 0;
    int got_frame = 0;
    int nb_frm_flushed = 0;
    struct buffer_data *bd = fmt_i_ctx->pb->opaque;
    //// enum AVMediaType stype = fmt_i_ctx->streams[stream_idx]->codecpar->codec_type;
    StreamContext *sctx = &bd->stream_ctx[stream_idx];
    if (!(sctx->enc_ctx->codec->capabilities & AV_CODEC_CAP_DELAY))
        return ret;
    do {
        //// ret = encode_write_frame_deprecated(stype, fmt_o_ctx, NULL, stream_idx, &got_frame);
        ret = encode_write_frame_v2(fmt_o_ctx, NULL, stream_idx, &got_frame);
        nb_frm_flushed++;
    } while((ret >= 0) && got_frame);
    av_log(NULL, AV_LOG_INFO, "Flushing %d frames on stream #%u encoder\n",
           nb_frm_flushed, stream_idx);
    return ret;
} // end of flush_encoder


static __attribute__((optimize("O0"))) int  try_filter_frame( AVFormatContext *fmt_i_ctx,
        AVFormatContext *fmt_o_ctx, AVFrame *frame, int stream_idx)
{
    int ret = 0;
    struct buffer_data *bd = fmt_i_ctx->pb->opaque;
    //// enum AVMediaType stype = fmt_i_ctx->streams[stream_idx]->codecpar->codec_type;
    StreamContext *sctx = &bd->stream_ctx[stream_idx];
    ret = av_buffersrc_add_frame_flags(sctx->filt_src_ctx, frame, 0);
    if (ret < 0) {
        av_log(NULL, AV_LOG_ERROR, "Error while feeding the filtergraph\n");
    }
    while (ret >= 0) {
        AVFrame *filt_frame = av_frame_alloc();
        if (!filt_frame) {
            ret = AVERROR(ENOMEM);
            break;
        }
        ret = av_buffersink_get_frame(sctx-> filt_sink_ctx, filt_frame);
        if (ret < 0) {
            av_log(NULL, AV_LOG_DEBUG, "Pulling filtered frame from filters, end\n");
            // if no more frames for output - returns AVERROR(EAGAIN)
            // if flushed and no more frames for output - returns AVERROR_EOF
            // rewrite retcode to 0 to show it as normal procedure completion
            if (ret == AVERROR(EAGAIN) || ret == AVERROR_EOF)
                ret = 0;
            av_frame_free(&filt_frame);
            break;
        }
        filt_frame->pict_type = AV_PICTURE_TYPE_NONE;
        //// ret = encode_write_frame_deprecated(stype, fmt_o_ctx, filt_frame, stream_idx, NULL);
        ret = encode_write_frame_v2(fmt_o_ctx, filt_frame, stream_idx, NULL);
        av_frame_free(&filt_frame);
        if (ret < 0) { break; }
    } // end of loop
    return ret;
} // end of try_filter_frame


static __attribute__((optimize("O0"))) int  try_decode_packet(
        AVFormatContext *fmt_i_ctx, AVFormatContext *fmt_o_ctx, AVPacket *pkt)
{
    int ret = 0;
    int stream_idx = pkt->stream_index;
    struct buffer_data *bd = fmt_i_ctx->pb->opaque;
    // enum AVMediaType stype = fmt_i_ctx->streams[stream_idx]->codecpar->codec_type;
    StreamContext *sctx = &bd->stream_ctx[stream_idx];
    assert (sctx->filter_graph);
    av_packet_rescale_ts(pkt, fmt_i_ctx->streams[stream_idx]->time_base,
            sctx->dec_ctx->time_base);
    int got_frame = 1;
    // int (*dec_func)(AVCodecContext *, AVFrame *, int *, const AVPacket *);
    // dec_func = (stype == AVMEDIA_TYPE_VIDEO) ? avcodec_decode_video2: avcodec_decode_audio4; // don't use deprecated functions
    // ret = dec_func(sctx->dec_ctx, frame, &got_frame, pkt);
    // if(!got_frame) {
    //     av_log(NULL, AV_LOG_DEBUG, "Got no frame from current packet, after reading %d bytes\n", ret);
    //     ret = 0;
    // }
    ret = avcodec_send_packet(sctx->dec_ctx, pkt);
    if (ret < 0) {
        av_log(NULL, AV_LOG_ERROR, "Failed to send packet to decoder, pos: 0x%08x size:%d \n", (uint32_t)pkt->pos, pkt->size);
    }
    AVFrame *frame = av_frame_alloc();
    while ((ret >= 0) && got_frame) {
        ret = avcodec_receive_frame(sctx->dec_ctx, frame);
        if (ret == AVERROR(EAGAIN) || ret == AVERROR_EOF) {
            got_frame = 0;
            ret = 0;
        } else if (ret >= 0) {
            frame->pts = frame->best_effort_timestamp;
            ret = try_filter_frame(fmt_i_ctx, fmt_o_ctx, frame, stream_idx);
        } else if (ret < 0) {
            av_log(NULL, AV_LOG_ERROR, "Failed to decode packet, pos: 0x%08x size:%d \n", (uint32_t)pkt->pos, pkt->size);
        }
    }
    av_frame_free(&frame);
    return ret;
} // end of try_decode_packet


static __attribute__((optimize("O0"))) uint8_t _recover_corrupted_packet(
        AVFormatContext *fmt_i_ctx, AVPacket *pkt)
{
    uint8_t done  = 0;
    struct buffer_data *bd = fmt_i_ctx->pb->opaque;
    StreamContext *sctx = &bd->stream_ctx[ pkt->stream_index ];
    AVStream *in_stream = fmt_i_ctx->streams[ pkt->stream_index ];
    if(!in_stream) { goto end; }
    int sample_idx = _av_stream_index_lookup(in_stream, pkt->pos, sctx->last_recovered_pkt_idx);
    if((sample_idx >= in_stream->nb_index_entries) || (sample_idx < 0)) {
        goto end;
    }
    AVIndexEntry *idx_entry = &in_stream->index_entries[ sample_idx ];
    if(!idx_entry) { goto end; }
    size_t expect_pkt_sz = idx_entry->size;
    size_t old_sz = pkt->size;
    size_t extra_sz = expect_pkt_sz - old_sz;
    if(av_grow_packet(pkt, extra_sz) < 0) {
        av_log(NULL, AV_LOG_ERROR,  "Failed to increease size for incomplete packet \n");
        goto end;
    }
    read(bd->fd, pkt->data + old_sz, extra_sz);
    fmt_i_ctx-> pb-> pos += extra_sz;
    fmt_i_ctx-> pb-> bytes_read += extra_sz;
    // lseek(bd->fd, -1 * extra_sz, SEEK_CUR);
    // fmt_i_ctx -> pb -> max_packet_size is unset
    sctx->last_recovered_pkt_idx = (size_t)sample_idx;
    done  = 1;
end:
    return done;
} // end of _recover_corrupted_packet


int  _start_reading_frames(AVFormatContext *fmt_i_ctx, AVFormatContext *fmt_o_ctx, size_t num_frame_read)
{
    int ret = 0, idx = 0;
    struct buffer_data *bd = fmt_i_ctx->pb->opaque;
    for (idx = 0; (!ret) && (idx < num_frame_read); idx++) {
        AVPacket packet = { .data = NULL, .size = 0 };
        ret = av_read_frame(fmt_i_ctx, &packet);
        if(ret < 0) {
            char errbuf[128];
            av_strerror(ret, &errbuf[0], 128);
            av_log(NULL, AV_LOG_ERROR,  "av_read_frame(), err = (%d)%s \n", ret, &errbuf[0]);
            continue;
        } else if (bd->size == 0) {
            uint8_t corruption_fixed = 0;
            if (avio_feof(fmt_i_ctx -> pb) && (packet.flags | AV_PKT_FLAG_CORRUPT)) {
                av_log(NULL, AV_LOG_WARNING, "av_read_frame(), corrupted packet, file pos: 0x%08x,"
                        " read packet size:%d \n", (uint32_t)packet.pos, packet.size);
                corruption_fixed = _recover_corrupted_packet(fmt_i_ctx, &packet);
            } // TODO set limit and report error in case of parsing extremely huge single packet
            bd->ptr  = bd->ptr_bak;
            bd->size = read(bd->fd, bd->ptr, bd->size_bak);
            if(bd->size == 0) {
                av_log(NULL, AV_LOG_WARNING, "av_read_frame(), end of input file reached (%d) \n", idx);
                idx = num_frame_read;
                goto end;
            }
            if (avio_feof(fmt_i_ctx -> pb)) {
                fmt_i_ctx -> pb -> eof_reached = 0;
                if(corruption_fixed) {
                    packet.flags &= (uint32_t) ~AV_PKT_FLAG_CORRUPT;
                } else {
                    goto end; // unrecoverable, abandon current packet
                }
            } // the fetched packet at this moment is corrupted, and should be abandoned
        }
        if(packet.stream_index >= fmt_i_ctx->nb_streams) {
            av_log(NULL, AV_LOG_ERROR,  "read packet, exceeding index, max_n_streams = %d, but got %d \n",
                    fmt_i_ctx->nb_streams, packet.stream_index);
            ret = AVERROR(ENOMEM);
            goto end;
        }
        av_log(NULL, AV_LOG_DEBUG, "fetched packet, pos: 0x%08x size:%d \n", (uint32_t)packet.pos, packet.size);
        ret = try_decode_packet(fmt_i_ctx, fmt_o_ctx, &packet);
end:
        av_packet_unref(&packet);
    } // end of read iteration
    // flush filters and encoders
    for (idx = 0; idx < fmt_i_ctx->nb_streams; idx++) {
        StreamContext *sctx = &bd->stream_ctx[idx];
        if (sctx->filter_graph) {
            ret = try_filter_frame(fmt_i_ctx, fmt_o_ctx, NULL, idx);
            if (ret < 0) {
                av_log(NULL, AV_LOG_ERROR, "Flushing filter failed on stream %d\n", idx);
                continue;
            }
        }
        ret = flush_encoder(fmt_i_ctx, fmt_o_ctx, idx);
        if (ret < 0) {
            av_log(NULL, AV_LOG_ERROR, "Flushing encoder failed on stream %d\n", idx);
        }
    }
    return ret;
} // end of _start_reading_frames

