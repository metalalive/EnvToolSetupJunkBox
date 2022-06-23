#ifndef APP_INPUT_FRAG_H
#define APP_INPUT_FRAG_H
#ifdef __cplusplus
extern "C" {
#endif

#include <libavformat/avformat.h>
#include <libavformat/avio.h>
#include "common.h"

void _app_input_deinit(AVFormatContext *fmt_ctx, struct buffer_data *bd);

AVFormatContext *_app_input_init(char *input_filename, struct buffer_data *bd);

#ifdef __cplusplus
} // end of extern C clause
#endif
#endif // end of  APP_INPUT_FRAG_H
