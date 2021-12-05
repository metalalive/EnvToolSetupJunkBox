#include <assert.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <fcntl.h>
#include <uv.h>
#define BUFFER_SIZE_BYTES 97

typedef struct context_s {
    struct {
        uv_pipe_t *_stdout;
        uv_pipe_t *_file;
    } pipe;
} context_t;


static void free_write_req(uv_write_t *req) {
    char *base = (char *) req->data;
    free((void *)base);
}

static void on_stdout_write(uv_write_t *req, int status) {
    free_write_req(req);
}

static void on_file_write(uv_write_t *req, int status) {
    free_write_req(req);
}


static void write_data(uv_pipe_t *dst, size_t sz, const uv_buf_t *rd_buf, uv_write_cb cb) {
    uv_write_t *req = (uv_write_t *)dst->data;
    char *base = (char *)malloc(sizeof(char) * sz);
    uv_buf_t lov = uv_buf_init(base, sz);
    req->data = (void *)base;
    memcpy(lov.base, rd_buf->base, sz);
    uv_write(req, (uv_stream_t *)dst, &lov, 1, cb);
}


static void alloc_buffer(uv_handle_t *handle, size_t suggested_sz, uv_buf_t *buf) {
    // handle == stdin_pipe
    // TODO, more efficient memory allocation plan
    char *base = (char *)malloc(sizeof(char) * suggested_sz);
    *buf = uv_buf_init(base, suggested_sz);
}

void on_read_stdin(uv_stream_t *stream, ssize_t nread, const uv_buf_t *buf) {
    context_t *ctx = (context_t *)stream->data;
    if (nread > 0) {
        write_data(ctx->pipe._stdout, nread, buf, on_stdout_write);
        write_data(ctx->pipe._file  , nread, buf, on_file_write);
    } else if (nread < 0) {
        // error status generated from macro UV_ERRNO_MAP()
        if(nread == UV_EOF) { // end-of-file in C/Linux Kernel, press Ctrl + d
            uv_close((uv_handle_t *)ctx->pipe._stdout, NULL);
            uv_close((uv_handle_t *)ctx->pipe._file  , NULL);
            uv_close((uv_handle_t *)stream, NULL);
        }
    }
    // ok to free buffer as write_data() copied the read content
    if(buf->base) {
        free((void *)buf->base);
    }
}


int main(int argc, char *argv[]) {
    assert(argc >= 1);
    int dst_path_idx = argc - 1;
    const char *dst_path = argv[dst_path_idx];
    int result = 0;

    uv_write_t req_wr_stdout;
    uv_write_t req_wr_file;
    uv_pipe_t stdin_pipe;
    uv_pipe_t stdout_pipe = {.data = (void *)&req_wr_stdout};
    uv_pipe_t file_pipe   = {.data = (void *)&req_wr_file};
    context_t ctx = {
        .pipe._stdout = &stdout_pipe,
        .pipe._file = &file_pipe
    };
    stdin_pipe.data = (void *)&ctx;

    uv_loop_t *loop = uv_default_loop();
    
    uv_pipe_init(loop, &stdin_pipe, 0);
    uv_pipe_open(&stdin_pipe, 0);
    uv_pipe_init(loop, &stdout_pipe, 0);
    uv_pipe_open(&stdout_pipe, 1);

    int flags = O_RDWR | O_CREAT; // rw+
    int mode = S_IRUSR | S_IWUSR | S_IRGRP;
    uv_fs_t file_req = {.file = 0};
    uv_file fd = (uv_file) uv_fs_open(loop, &file_req, dst_path, flags, mode, NULL);
    uv_pipe_init(loop, &file_pipe, 0);
    uv_pipe_open(&file_pipe, fd);

    uv_read_start((uv_stream_t *)&stdin_pipe, alloc_buffer, on_read_stdin);
    uv_run(loop, UV_RUN_DEFAULT);

    result = uv_loop_close(loop);
    assert(result == 0);
    uv_fs_req_cleanup(&file_req);
    return 0;
}
