#include <stdint.h>
#include <stdlib.h>
#include "lzrw.h"

typedef unsigned char BYTE;

BYTE* lzrw3a_c(int action, BYTE* buffer, size_t size, int* pSizeOut) {
    uint32_t sizeOut = 0;
    uint32_t workingMemoryLen = lzrw3_req_mem();

    // allocate working memory
    BYTE* workingMemory = malloc(sizeof *workingMemory * workingMemoryLen);
    if (!workingMemory) {
        return NULL;
    }

    // allocate result buffer
    BYTE* bufferOut = malloc(sizeof *buffer * size * 10);
    if (!bufferOut) {
        free(workingMemory);
        return NULL;
    }

    // compress / decompress buffer
    lzrw3a_compress(action, workingMemory, buffer, size, bufferOut, &sizeOut);
    *pSizeOut = sizeOut;

    // cleanup
    free(workingMemory);
    return bufferOut;
}
