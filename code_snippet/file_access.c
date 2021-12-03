#include <assert.h>
#include <stdio.h>
#include <stdlib.h>
#include <fcntl.h>
#include <uv.h>
#define BUFFER_SIZE_BYTES 97

typedef struct context_s {
    char *rd_buf;
    uv_fs_t  *src_req;
    uv_fs_t  *dst_req;
} context_t;

void on_read(uv_fs_t* req);


void on_close(uv_fs_t* req) {
    fprintf(stdout, "closed file (fd=%d) \n", (int)req->file);
}

void on_write(uv_fs_t* req) {
    int result = 0;
    unsigned int num_bufs = 1;
    if (req->result > 0) {
        context_t *ctx = (context_t *)req->data;
        uv_fs_t *src_req = ctx->src_req;
        uv_buf_t lov = src_req->bufsml[0];
        lov.len = BUFFER_SIZE_BYTES;
        result = uv_fs_read(req->loop, src_req, (uv_file)src_req->file, &lov, num_bufs, -1, on_read);
        assert(result == 0);
    } else {
        fprintf(stderr, "error when writing file: %s \n", uv_strerror((int)req->result));
    }
}


void on_read(uv_fs_t* req) {
    int result = 0;
    context_t *ctx = (context_t *)req->data;
    uv_fs_t *dst_req = ctx->dst_req;
    if (req->result > 0) {
        // the read buffer (specified by previous callback) is located in internal member `bufsml` of request
        uv_buf_t lov = req->bufsml[0];
        assert(req->result <= lov.len);
        if (req->result < lov.len) {
            lov.len = req->result;
        }
        // the read buffer should still exist
        result = uv_fs_write(req->loop, dst_req, (uv_file)dst_req->file, &lov, 1, -1, on_write);
        assert(result == 0);
    } else if (req->result == 0) {
        // reuse the same request and loop instance again
        // close input file (synchronously if not providing callback), then open output file
        ////result = uv_fs_close(req->loop, req, req->file, NULL);
        result = uv_fs_close(req->loop, req, req->file, on_close); // will cause segmentation fault
        assert(result == 0);
        result = uv_fs_close(req->loop, dst_req, dst_req->file, on_close);
        assert(result == 0);
    } else {
        fprintf(stderr, "error when reading file: %s \n", uv_strerror((int)req->result));
    }
}

void on_open(uv_fs_t* req) {
    const char* filepath = req->path;
    if (req->result >= 0) { // valid result should be file descriptor in this case
        int result = 0;
        unsigned int num_bufs = 1;
        context_t *ctx = (context_t *)req->data;
        req->file = (uv_file)req->result;
        // you never know which one comes prior to the other, src_req or dst_req
        if (ctx->src_req->file && ctx->dst_req->file) {
            uv_buf_t lov = uv_buf_init(ctx->rd_buf, BUFFER_SIZE_BYTES);
            // reuse the same request and loop instance
            // why using off_t ? why off is set to -1 ? does it relate to file offset ?
            result = uv_fs_read(req->loop, ctx->src_req, ctx->src_req->file, &lov, num_bufs, -1, on_read);
            assert(result == 0);
        }
    } else {
        fprintf(stderr, "error when opening file %s \n", uv_strerror((int)req->result));
    }
    // libuv expect each fs request is used for only one operation, if you
    // want to reuse the request, make sure deallocate the space pointed by `path` member
    if (filepath != NULL) {
        free((void *)filepath);
    }
}

// This example demonstrates file access sequence, firstly it read part of bytes from
// given input file to buffer, then it writes the bytes in the buffer to given output file
int main(int argc, char *argv[]) {
    assert(argc >= 2);
    int src_path_idx = argc - 2;
    int dst_path_idx = argc - 1;
    const char *src_path = argv[src_path_idx];
    const char *dst_path = argv[dst_path_idx];
    int result = 0;

    uv_loop_t *loop = (uv_loop_t *) malloc(sizeof(uv_loop_t));
    result = uv_loop_init(loop);
    assert(result == 0);

    uv_fs_t  src_req = {.file = 0};
    uv_fs_t  dst_req = {.file = 0};
    context_t ctx = {
        .rd_buf = (char *)malloc(sizeof(char) * BUFFER_SIZE_BYTES) ,
        .src_req = &src_req,
        .dst_req = &dst_req
    };
    src_req.data = (void *)&ctx;
    dst_req.data = (void *)&ctx;
    result = uv_fs_open(loop, &src_req, src_path, O_RDONLY, 0, on_open);
    assert(result == 0);
    int flags = O_WRONLY | O_CREAT;
    result = uv_fs_open(loop, &dst_req, dst_path, flags, 0, on_open);
    assert(result == 0);

    uv_run(loop, UV_RUN_DEFAULT);

    result = uv_loop_close(loop);
    assert(result == 0);
    uv_fs_req_cleanup(&src_req);
    uv_fs_req_cleanup(&dst_req);
    free((void *)ctx.rd_buf);
    free((void *)loop);
    return 0;
} // end of main()

