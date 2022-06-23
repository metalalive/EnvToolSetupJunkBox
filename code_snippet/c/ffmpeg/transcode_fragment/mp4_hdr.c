#define _BSD_SOURCE
#include <endian.h>
#include <stdint.h>
#include <stdlib.h>
#include <unistd.h>
#include <libavutil/common.h>
#include <libavutil/error.h>

#include "mp4_hdr.h"

typedef struct MOVAtom {
    uint32_t size;
    uint32_t type;
} MOVAtom;

static size_t _filechunk_copy(int dst_fd, int src_fd, size_t req_sz)
{
#define BUF_SZ  128
    size_t n_remain = req_sz;
    size_t n_rd = 0; 
    char buf[BUF_SZ] = {0};
    do {
        size_t n_cp = FFMIN(n_remain, BUF_SZ);
        n_rd = read(src_fd, &buf[0], n_cp);
        write(dst_fd, &buf[0], n_rd);
        n_remain -= n_rd;
    } while((n_remain > 0) && (n_rd > 0));
    return  req_sz - n_remain;
#undef  BUF_SZ
}

int  serialize_mp4_header(int dst_fd, int src_fd)
{ // always ensure `mdat` atom is at the end of file
    MOVAtom a0 = {0}, a1 = {0}, mdat = {0};
    int mdat_pos = 0;
    int eof_reached = 0;
    lseek(dst_fd, 0, SEEK_SET);
    lseek(src_fd, 0, SEEK_SET);
    while(!eof_reached) {
        size_t nread = read(src_fd, &a0, sizeof(a0));
        eof_reached = nread == 0;
        if(eof_reached)
            continue;
        a1 = (MOVAtom) {.size=htobe32(a0.size), .type=a0.type};
        uint8_t is_mdat = strncmp((char *)&a1.type, "mdat", 4) == 0;
        if(is_mdat) { // skip, jump to next atom
            mdat_pos = lseek(src_fd, 0, SEEK_CUR);
            mdat = a0;
            lseek(src_fd, a1.size - sizeof(a1), SEEK_CUR);
        } else { // copy entire atom
            lseek(src_fd, -1 * sizeof(a0), SEEK_CUR);
            size_t nread = _filechunk_copy(dst_fd, src_fd, a1.size);
            if(nread != a1.size) {
                return ENOMEM;
            }
        }
    } // end of loop
    if(mdat.size > 0) {
        write(dst_fd, (char *)&mdat, sizeof(mdat));
    } else {
        return ENOMEM;
    }
    return mdat_pos;
} // end of serialize_mp4_header

