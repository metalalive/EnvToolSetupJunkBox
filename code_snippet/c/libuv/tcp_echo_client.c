#include <assert.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <uv.h>
#define BUFFER_SIZE_BYTES 17

typedef struct {
    uv_stream_t *dst; // could be uv_tcp_t or uv_pipe_t
    uv_write_t  *flush_req;
    uv_stream_t *local_input;
    struct {
        size_t  sz;
        char   *ptr;
    } buf;
} context_t;

static void on_stdout_write(uv_write_t *req, int status) {
    if(status < 0) {
        fprintf(stderr, "Failed to write bytes to stdout, reason: %s \n", uv_strerror(status));
    }
    uv_pipe_t *stdout_pipe = (uv_pipe_t *)req->handle;
    uv_close((uv_handle_t *)stdout_pipe, NULL);
}


static void on_sock_read(uv_stream_t *socket, ssize_t nread, const uv_buf_t *buf) {
    context_t *ctx = (context_t *)socket->data;
    if (nread > 0) {
        uv_pipe_t *stdout_pipe = (uv_pipe_t *)ctx->dst;
        uv_pipe_init(socket->loop, stdout_pipe, 0);
        uv_pipe_open(stdout_pipe, 1); // default file descriptor  for stdout
        uv_write(ctx->flush_req, ctx->dst, buf, 1, on_stdout_write);
    } else if (nread < 0) {
        fprintf(stderr, "Read error on client, reason: %s \n", uv_strerror(nread));
        uv_close((uv_handle_t *)ctx->local_input, NULL);
        uv_close((uv_handle_t *)socket, NULL);
    }
}

static void alloc_buffer(uv_handle_t *handle, size_t suggested_sz, uv_buf_t *buf) {
    context_t *ctx = (context_t *) handle->data;
    size_t  curr_sz = ctx->buf.sz;
    char   *curr_base = ctx->buf.ptr;
    *buf = uv_buf_init(curr_base, curr_sz);
}

static void on_sock_write(uv_write_t *req, int status) {
    uv_stream_t *socket = req->handle;
    if (status >= 0) {
        uv_read_start(socket, alloc_buffer, on_sock_read);
    } else {
        fprintf(stderr, "Failed to write bytes to the socket, reason: %s \n", uv_strerror(status));
        context_t *ctx = (context_t *)socket->data;
        uv_close((uv_handle_t *)ctx->local_input, NULL);
        uv_close((uv_handle_t *)socket, NULL);
    }
}


static void on_stdin_read(uv_stream_t *stdin_pipe, ssize_t nread, const uv_buf_t *buf) {
    context_t *ctx = (context_t *)stdin_pipe->data;
    if (nread > 0) {
        buf->base[nread++] = (char)0;
        assert(buf->len >= nread);
        uv_write(ctx->flush_req, ctx->dst, buf, 1, on_sock_write);
    } else if (nread < 0) {
        if(nread == UV_EOF) { // end-of-file in C/Linux Kernel, press Ctrl + d
            fprintf(stdout, "End of streaming, closing connection ...\n");
        } else {
            fprintf(stderr, "Read error on client, reason: %s \n", uv_strerror(nread));
        }
        uv_close((uv_handle_t *)ctx->dst, NULL);
        uv_close((uv_handle_t *)stdin_pipe, NULL);
    }
}

static void on_connect(uv_connect_t *req, int status) {
    if (status >= 0) {
        uv_pipe_t *stdin_pipe = (uv_pipe_t *) req->data;
        uv_pipe_open(stdin_pipe, 0); // file descriptor is 0
        uv_read_start((uv_stream_t *)stdin_pipe, alloc_buffer, on_stdin_read);
    } else {
        fprintf(stderr, "TCP connection error, %s \n", uv_strerror(status));
        uv_close((uv_handle_t *)req->handle, NULL);
    }
}


int main(int argc, char *argv[]) {
    assert(argc >= 1);
    int ip_idx = argc - 2;
    int port_idx = argc - 1;
    const char *ip   = argv[ip_idx];
    int port = (int) strtol(argv[port_idx], (char **)NULL, 10);
    int result = 0;
    char tx_buffer[BUFFER_SIZE_BYTES];
    char rx_buffer[BUFFER_SIZE_BYTES];

    struct sockaddr_in addr;
    uv_tcp_t     socket;
    uv_connect_t conn_req;
    uv_write_t   flush_req;
    uv_pipe_t    stdin_pipe, stdout_pipe;

    uv_loop_t *loop = uv_default_loop();
    uv_ip4_addr(ip, port, &addr);
    uv_tcp_init(loop, &socket);
    uv_pipe_init(loop, &stdin_pipe, 0);
    context_t tx_ctx = {
        .local_input = (uv_stream_t *)&stdin_pipe,
        .dst = (uv_stream_t *)&socket,  .flush_req  = &flush_req,
        .buf.sz  = BUFFER_SIZE_BYTES,   .buf.ptr = (char *)&tx_buffer,
    };
    context_t rx_ctx = {
        .local_input = (uv_stream_t *)&stdin_pipe,
        .dst = (uv_stream_t *)&stdout_pipe,  .flush_req  = &flush_req,
        .buf.sz  = BUFFER_SIZE_BYTES,   .buf.ptr = (char *)&rx_buffer,
    };
    stdin_pipe.data = (void *) &tx_ctx;
    socket.data     = (void *) &rx_ctx;
    conn_req.data  = (void *) &stdin_pipe;
    result = uv_tcp_connect(&conn_req, &socket, (const struct sockaddr *)&addr, on_connect);
    assert(result == 0);

    uv_run(loop, UV_RUN_DEFAULT);

    result = uv_loop_close(loop);
    assert(result == 0);
    return 0;
}
