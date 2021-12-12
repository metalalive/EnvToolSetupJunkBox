#include <assert.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <uv.h>
#define  NUM_ROUNDS  20

typedef struct {
    uv_barrier_t *blocker;
    uv_rwlock_t  *lock;
    int          *shared_num;
    uint8_t       id;
} context_t;


static void reader_entry(void* arg) {
    context_t *ctx = (context_t *)arg;
    uint8_t idx = 0;
    for(idx = 0; idx < NUM_ROUNDS; idx++) {
        //fprintf(stdout, "Reader %d: lock \n", ctx->id);
        uv_rwlock_rdlock(ctx->lock);
        int shr_num = *ctx->shared_num;
        fprintf(stdout, "Reader %d: got shared num %d \n", ctx->id, shr_num);
        uv_rwlock_rdunlock(ctx->lock);
        uv_sleep(1);
        //fprintf(stdout, "Reader %d: unlock \n", ctx->id);
    }
    uv_barrier_wait(ctx->blocker);
}

static void writer_entry(void* arg) {
    context_t *ctx = (context_t *)arg;
    uint8_t idx = 0;
    for(idx = 0; idx < NUM_ROUNDS; idx++) {
        //fprintf(stdout, "Writer %d: lock \n", ctx->id);
        uv_rwlock_wrlock(ctx->lock);
        (*ctx->shared_num)++;
        fprintf(stdout, "Writer %d: incremented shared num %d \n", ctx->id, *ctx->shared_num);
        uv_rwlock_wrunlock(ctx->lock);
        uv_sleep(1);
        //fprintf(stdout, "Writer %d: unlock \n", ctx->id);
    }
    uv_barrier_wait(ctx->blocker);
}


int main(int argc, char *argv[]) {
    uint8_t num_readers = 2;
    uint8_t num_writers = 1;
    uint8_t num_extra_threads = num_readers + num_writers;
    uint8_t idx = 0;
    // Barrier in software is typically used to wait a number of threads
    // (usually specified in advance) to complete their execution before
    // any particular thread can proceed forward
    uv_barrier_t blocker;
    uv_rwlock_t  lock;
    int shared_num = 0;
    uv_thread_t  threads[num_extra_threads];
    context_t    ctx_list[num_extra_threads];
    uv_thread_cb th_entry_fns[num_extra_threads];
    for(idx = 0; idx < num_readers; idx++) {
        th_entry_fns[idx] = reader_entry;
    }
    for(idx = num_readers; idx < num_extra_threads; idx++) {
        th_entry_fns[idx] = writer_entry;
    }

    // the barrier is used for at least 4 threads reaching the control point specified in their program execution (3 extra threads forked from main thread)
    uv_barrier_init(&blocker, (unsigned int)(1 + num_extra_threads));
    uv_rwlock_init(&lock);

    for(idx = 0; idx < num_extra_threads; idx++) {
        context_t *ctx = &ctx_list[idx];
        ctx->blocker = &blocker;
        ctx->lock = &lock;
        ctx->shared_num = &shared_num;
        ctx->id = idx;
        uv_thread_create(&threads[idx], th_entry_fns[idx], (void *)ctx);
    }
    // main thread start waiting until all forked threads have reached their control points specified by the barrier
    uv_barrier_wait(&blocker);
    uv_barrier_destroy(&blocker);
    uv_rwlock_destroy(&lock);
    // pthread relies on join() to free up the space allocated by create()
    for(idx = 0; idx < num_extra_threads; idx++) {
        uv_thread_join(&threads[idx]);
    }
    return 0;
}
