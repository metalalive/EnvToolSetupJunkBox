/**
 *  extended from example avio_reading.c and transcoding.c
 */

#include "input_frag.h"
#include "output_hls.h"
#include "filter.h"
#include "packet.h"


int main(int argc, char *argv[])
{
    if (argc != 4) {
        fprintf(stderr, "usage: %s <input_file> <output_file> <num_frames> \n"
                "API example program to show how to transcode and read from a custom buffer "
                "accessed through a custom AVIOContext.\n", argv[0]);
        return 1;
    }
    size_t num_frames = strtoul(argv[3], NULL, 10);
    if(num_frames == 0) {
        fprintf(stderr, "Failed to convert num_frames from `%s` \n", argv[3]);
        return 1;
    }
    int ret = 0;
    /* fill opaque structure used by the AVIOContext read callback */
    struct buffer_data i_bd = {0}, o_bd = {0};
    AVFormatContext *ifmt_ctx = _app_input_init(argv[1], &i_bd);
    AVFormatContext *ofmt_ctx = _app_output_init(argv[2], &o_bd, ifmt_ctx);
    if (!ifmt_ctx || !ofmt_ctx) {
        ret = AVERROR(ENOMEM);
        goto end;
    }
    ret = init_filters(ifmt_ctx);
    if (ret) { goto end; }
    ret = start_processing_packets(ifmt_ctx, ofmt_ctx, num_frames);
    av_write_trailer(ofmt_ctx);
    o_bd.flush_output(&o_bd);
end:
    _app_input_deinit(ifmt_ctx, &i_bd);
    _app_output_deinit(ofmt_ctx, &o_bd);
    if (ret < 0) {
        char errbuf[128];
        av_strerror(ret, &errbuf[0], 128);
        av_log(NULL, AV_LOG_ERROR, "Error occurred: %s\n", av_err2str(ret));
    }
    return ret;
}
