#include <stdint.h>
#include <stdio.h>
#include <assert.h>

int32_t  buggy_cross_product (int8_t *vec1, const int8_t *vec2,
	uint8_t num_cols)
{
    int result = 0;
    assert(vec1 != NULL);
    assert(vec2 != NULL);
    for(uint8_t idx = 0; idx < num_cols; idx++) {
	result += vec1[idx]-- * vec2[idx];
    }
    return result;
}
