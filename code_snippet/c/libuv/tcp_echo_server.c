#include <assert.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <uv.h>

#define DEFAULT_BACKLOG 0x80
// initial buffer size in bytes
#define BUFFER_SIZE_BYTES 17

typedef struct {
    struct {
        size_t  sz;
        char   *ptr;
    } buf;
} client_ctx_t;


static void on_close_conn(uv_handle_t *handle) {
    if (handle->data != NULL) {
        client_ctx_t *ctx = (client_ctx_t *) handle->data;
        if (ctx->buf.ptr != NULL) {
            free((void *)ctx->buf.ptr);
            ctx->buf.ptr = NULL;
        }
        free(handle->data);
        handle->data = NULL;
    }
    free((void *)handle);
}


static void echo_on_write(uv_write_t *req, int status) {
    if(status) {
        fprintf(stderr, "Write error on client, reason: %s \n", uv_strerror(status));
    } else {
        fprintf(stdout, "echo back to the client successfully ...\n");
    }
    free(req->data);
    free((void *)req);
}

static void echo_on_read(uv_stream_t *client, ssize_t nread, const uv_buf_t *rd_buf) {
    if(nread > 0) {  // TODO, optimization
        fprintf(stdout, "Received content from client : %s \n", rd_buf->base);
        uv_write_t *req = (uv_write_t *) malloc(sizeof(uv_write_t));
        char *base = (char *)malloc(sizeof(char) * nread);
        uv_buf_t wr_buf = uv_buf_init(base, nread);
        req->data = (void *)base;
        memcpy(wr_buf.base, rd_buf->base, nread);
        uv_write(req, client, &wr_buf, 1, echo_on_write);
    } else if (nread < 0) {
        // error status generated from macro UV_ERRNO_MAP()
        if(nread == UV_EOF) { // end-of-file in C/Linux Kernel, press Ctrl + d
            fprintf(stdout, "End of streaming, closing connection ...\n");
        } else {
            fprintf(stderr, "Read error on client, reason: %s \n", uv_strerror(nread));
        }
        uv_close((uv_handle_t *)client, on_close_conn);
    }
}


static void alloc_buffer(uv_handle_t *client, size_t suggested_sz, uv_buf_t *buf) {
    client_ctx_t *ctx = (client_ctx_t *) client->data;
    size_t  curr_sz = ctx->buf.sz;
    char   *curr_base = ctx->buf.ptr;
    *buf = uv_buf_init(curr_base, curr_sz);
}

static void on_new_conn(uv_stream_t *server, int status) {
    if (status < 0) {
        fprintf(stderr, "TCP connection error, %s \n", uv_strerror(status));
        return;
    }
    int result = 0;
    uv_tcp_t *client = (uv_tcp_t *) malloc(sizeof(uv_tcp_t));
    client->data = NULL;
    // may also process the client request in different loop / thread
    uv_tcp_init(server->loop, client);
    result = uv_accept(server, (uv_stream_t *)client);
    if (result == 0) {
        client_ctx_t *ctx = (client_ctx_t *) malloc(sizeof(client_ctx_t));
        ctx->buf.sz  = (size_t)BUFFER_SIZE_BYTES;
        ctx->buf.ptr = (char *)malloc(sizeof(char) * BUFFER_SIZE_BYTES);
        client->data = (void *)ctx;
        fprintf(stdout, "New client connected ... \n");
        uv_read_start((uv_stream_t *)client, alloc_buffer, echo_on_read);
    } else {
        uv_close((uv_handle_t *)client, on_close_conn);
    }
}

int main(int argc, char *argv[]) {
    assert(argc >= 1);
    int ip_idx = argc - 2;
    int port_idx = argc - 1;
    const char *ip   = argv[ip_idx];
    int port = (int) strtol(argv[port_idx], (char **)NULL, 10);
    int result = 0;

    struct sockaddr_in addr;
    uv_tcp_t server;
    uv_loop_t *loop = uv_default_loop();
    uv_tcp_init(loop, &server);
    uv_ip4_addr(ip, port, &addr);
    uv_tcp_bind(&server, (const struct sockaddr *)&addr, 0);
    result = uv_listen((uv_stream_t *)&server, DEFAULT_BACKLOG, on_new_conn);
    if (result) {
        fprintf(stderr, "Listen error %s \n", uv_strerror(result));
        return 1;
    }
    uv_run(loop, UV_RUN_DEFAULT);

    result = uv_loop_close(loop);
    assert(result == 0);
    return 0;
}
