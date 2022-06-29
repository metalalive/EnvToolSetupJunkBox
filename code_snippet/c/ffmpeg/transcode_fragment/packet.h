#ifndef APP_PACKET_H
#define APP_PACKET_H
#ifdef __cplusplus
extern "C" {
#endif

#include <libavformat/avformat.h>

int  start_processing_packets(AVFormatContext *fmt_i_ctx, AVFormatContext *fmt_o_ctx, size_t num_pkt_rd);

#ifdef __cplusplus
} // end of extern C clause
#endif
#endif // end of  APP_PACKET_H
