#ifndef APP_FILTER_H
#define APP_FILTER_H
#ifdef __cplusplus
extern "C" {
#endif

#include <libavformat/avformat.h>

int init_filters(AVFormatContext *fmt_i_ctx);

#ifdef __cplusplus
} // end of extern C clause
#endif
#endif // end of  APP_FILTER_H
