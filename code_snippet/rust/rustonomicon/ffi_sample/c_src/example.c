#include <stdint.h>
#include <stdio.h>
#include <ctype.h>

typedef struct {
    struct {
        uint8_t  cwr:1; // bit 0
        uint8_t  ece:1;
        uint8_t  urg:1;
        uint8_t  ack:1;
        uint8_t  psh:1;
        uint8_t  rst:1;
        uint8_t  syn:1;
        uint8_t  fin:1; // bit 7
    } flags;
    unsigned short timeout_s;
    uint8_t     qos;
    uint8_t     fastopen;
    struct {
	char   *data;
	size_t  len;
    } payld;
} lowlvl_pkt_t;

int32_t  censor_packet(lowlvl_pkt_t *p)
{
    int32_t result = 0;
    p->flags.cwr = ~ p->flags.cwr;
    p->flags.ece = ~ p->flags.ece;
    p->flags.urg = ~ p->flags.urg;
    p->flags.ack = ~ p->flags.ack;
    p->fastopen  <<= 1;
    size_t idx = 0, num_skipped = 0;
    for(idx = 0; idx < p->payld.len; idx++) {
	char c = p->payld.data[idx];
	if(isalnum((int)c)) {
	    p->payld.data[idx - num_skipped] = c;
	} else {
	    num_skipped++;
	}
    }
    p->payld.len -= num_skipped;
    // if(num_skipped > 0)
    //     p->payld.data[p->payld.len] = 0x0;
    return result;
}

