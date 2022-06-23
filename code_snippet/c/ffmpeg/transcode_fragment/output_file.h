#ifndef APP_OUTPUT_FILE_H
#define APP_OUTPUT_FILE_H
#ifdef __cplusplus
extern "C" {
#endif

#include <libavformat/avformat.h>
#include <libavformat/avio.h>
#include "common.h"

void _app_output_deinit(AVFormatContext *fmt_ctx, struct buffer_data *bd);

AVFormatContext *_app_output_init(char *filename, struct buffer_data *bd, AVFormatContext *fmt_in_ctx);

#ifdef __cplusplus
} // end of extern C clause
#endif
#endif // end of  APP_OUTPUT_FILE_H
