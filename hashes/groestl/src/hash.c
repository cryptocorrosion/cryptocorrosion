/* hash.h     Aug 2011
 *
 * Groestl implementation for different versions.
 * Author: Krystian Matusiewicz, Günther A. Roland, Martin Schläffer
 *
 * This code is placed in the public domain
 */

#include <stdint.h>

#define LENGTH 256

/* some sizes (number of bytes) */
#define ROWS 8
#define LENGTHFIELDLEN ROWS
#define COLS512 8
#define SIZE512 (ROWS*COLS512)
#define ROUNDS512 10

#define COLS COLS512
#define SIZE SIZE512
#define ROUNDS ROUNDS512

#define ROTL64(a,n) ((((a)<<(n))|((a)>>(64-(n))))&0xffffffffffffffffull)

#define EXT_BYTE(var,n) ((u8)((uint64_t)(var) >> (8*n)))
#define U64BIG(a) \
  ((ROTL64(a, 8) & 0x000000FF000000FFull) | \
   (ROTL64(a,24) & 0x0000FF000000FF00ull) | \
   (ROTL64(a,40) & 0x00FF000000FF0000ull) | \
   (ROTL64(a,56) & 0xFF000000FF000000ull))

/* NIST API begin */
typedef unsigned char BitSequence;
typedef unsigned int DataLength;
typedef enum { SUCCESS = 0, FAIL = 1 } HashReturn;
typedef struct {
  __attribute__ ((aligned (32))) uint64_t chaining[SIZE/8];      /* actual state */
  __attribute__ ((aligned (32))) BitSequence buffer[SIZE];  /* data buffer */
  uint64_t block_counter;        /* message block counter */
  int buf_ptr;              /* data buffer pointer */
  int bits_in_last_byte;    /* no. of message bits in last byte of
                               data buffer */
} hashState;

void groestl_init(hashState*);
HashReturn groestl_update(hashState*, const BitSequence*, DataLength);
void groestl_final(hashState*, BitSequence*);
/* NIST API end   */

/* groestl-asm-aes.h     Aug 2011
 *
 * Groestl implementation with inline assembly using ssse3, sse4.1, and aes
 * instructions.
 * Authors: Günther A. Roland, Martin Schläffer, Krystian Matusiewicz
 *
 * This code is placed in the public domain
 */

/* global constants  */
__attribute__ ((aligned (16))) static const uint64_t ROUND_CONST_Lx[] = {
  0x0000000000000000ULL,  0xffffffffffffffffULL };
__attribute__ ((aligned (16))) static const uint64_t ROUND_CONST_L0[] = {
  (0ULL * 0x0101010101010101ULL)  ^ 0x7060504030201000ULL,
  0xffffffffffffffffULL,
  (1ULL * 0x0101010101010101ULL)  ^ 0x7060504030201000ULL,
  0xffffffffffffffffULL,
  (2ULL * 0x0101010101010101ULL)  ^ 0x7060504030201000ULL,
  0xffffffffffffffffULL,
  (3ULL * 0x0101010101010101ULL)  ^ 0x7060504030201000ULL,
  0xffffffffffffffffULL,
  (4ULL * 0x0101010101010101ULL)  ^ 0x7060504030201000ULL,
  0xffffffffffffffffULL,
  (5ULL * 0x0101010101010101ULL)  ^ 0x7060504030201000ULL,
  0xffffffffffffffffULL,
  (6ULL * 0x0101010101010101ULL)  ^ 0x7060504030201000ULL,
  0xffffffffffffffffULL,
  (7ULL * 0x0101010101010101ULL)  ^ 0x7060504030201000ULL,
  0xffffffffffffffffULL,
  (8ULL * 0x0101010101010101ULL)  ^ 0x7060504030201000ULL,
  0xffffffffffffffffULL,
  (9ULL * 0x0101010101010101ULL)  ^ 0x7060504030201000ULL,
  0xffffffffffffffffULL,
};
__attribute__ ((aligned (16))) static const uint64_t ROUND_CONST_L7[] = {
  0x0000000000000000ULL,
  (0ULL * 0x0101010101010101ULL)  ^ 0x8f9fafbfcfdfefffULL,
  0x0000000000000000ULL,
  (1ULL * 0x0101010101010101ULL)  ^ 0x8f9fafbfcfdfefffULL,
  0x0000000000000000ULL,
  (2ULL * 0x0101010101010101ULL)  ^ 0x8f9fafbfcfdfefffULL,
  0x0000000000000000ULL,
  (3ULL * 0x0101010101010101ULL)  ^ 0x8f9fafbfcfdfefffULL,
  0x0000000000000000ULL,
  (4ULL * 0x0101010101010101ULL)  ^ 0x8f9fafbfcfdfefffULL,
  0x0000000000000000ULL,
  (5ULL * 0x0101010101010101ULL)  ^ 0x8f9fafbfcfdfefffULL,
  0x0000000000000000ULL,
  (6ULL * 0x0101010101010101ULL)  ^ 0x8f9fafbfcfdfefffULL,
  0x0000000000000000ULL,
  (7ULL * 0x0101010101010101ULL)  ^ 0x8f9fafbfcfdfefffULL,
  0x0000000000000000ULL,
  (8ULL * 0x0101010101010101ULL)  ^ 0x8f9fafbfcfdfefffULL,
  0x0000000000000000ULL,
  (9ULL * 0x0101010101010101ULL)  ^ 0x8f9fafbfcfdfefffULL,
};
__attribute__ ((aligned (16))) static const uint64_t TRANSP_MASK[] = {
  0x0d0509010c040800ULL, 0x0f070b030e060a02ULL };
__attribute__ ((aligned (16))) static const uint64_t SUBSH_MASK[] = {
  0x0c0f0104070b0e00ULL,
  0x03060a0d08020509ULL,
  0x0e090205000d0801ULL,
  0x04070c0f0a03060bULL,
  0x080b0306010f0a02ULL,
  0x05000e090c04070dULL,
  0x0a0d040702090c03ULL,
  0x0601080b0e05000fULL,
  0x0b0e0500030a0d04ULL,
  0x0702090c0f060108ULL,
  0x0d080601040c0f05ULL,
  0x00030b0e0907020aULL,
  0x0f0a0702050e0906ULL,
  0x01040d080b00030cULL,
  0x090c000306080b07ULL,
  0x02050f0a0d01040eULL,
};
__attribute__ ((aligned (16))) static const uint64_t ALL_1B[] = {
  0x1b1b1b1b1b1b1b1bULL, 0x1b1b1b1b1b1b1b1bULL };


#define tos(a)    #a
#define tostr(a)  tos(a)


/* xmm[i] will be multiplied by 2
 * xmm[j] will be lost
 * xmm[k] has to be all 0x1b */
#define MUL2(i, j, k){\
  asm("pxor xmm"tostr(j)", xmm"tostr(j)"");\
  asm("pcmpgtb xmm"tostr(j)", xmm"tostr(i)"");\
  asm("paddb xmm"tostr(i)", xmm"tostr(i)"");\
  asm("pand xmm"tostr(j)", xmm"tostr(k)"");\
  asm("pxor xmm"tostr(i)", xmm"tostr(j)"");\
}/**/

/* Yet another implementation of MixBytes.
   This time we use the formulae (3) from the paper "Byte Slicing Groestl".
   Input: a0, ..., a7
   Output: b0, ..., b7 = MixBytes(a0,...,a7).
   but we use the relations:
   t_i = a_i + a_{i+3}
   x_i = t_i + t_{i+3}
   y_i = t_i + t+{i+2} + a_{i+6}
   z_i = 2*x_i
   w_i = z_i + y_{i+4}
   v_i = 2*w_i
   b_i = v_{i+3} + y_{i+4}
   We keep building b_i in registers xmm8..xmm15 by first building y_{i+4} there
   and then adding v_i computed in the meantime in registers xmm0..xmm7.
   We almost fit into 16 registers, need only 3 spills to memory.
   This implementation costs 7.7 c/b giving total speed on SNB: 10.7c/b.
   K. Matusiewicz, 2011/05/29 */
#define MixBytes(a0, a1, a2, a3, a4, a5, a6, a7, b0, b1, b2, b3, b4, b5, b6, b7){\
  /* t_i = a_i + a_{i+1} */\
  asm("movdqa xmm"tostr(b6)", xmm"tostr(a0)"");\
  asm("movdqa xmm"tostr(b7)", xmm"tostr(a1)"");\
  asm("pxor xmm"tostr(a0)", xmm"tostr(a1)"");\
  asm("movdqa xmm"tostr(b0)", xmm"tostr(a2)"");\
  asm("pxor xmm"tostr(a1)", xmm"tostr(a2)"");\
  asm("movdqa xmm"tostr(b1)", xmm"tostr(a3)"");\
  asm("pxor xmm"tostr(a2)", xmm"tostr(a3)"");\
  asm("movdqa xmm"tostr(b2)", xmm"tostr(a4)"");\
  asm("pxor xmm"tostr(a3)", xmm"tostr(a4)"");\
  asm("movdqa xmm"tostr(b3)", xmm"tostr(a5)"");\
  asm("pxor xmm"tostr(a4)", xmm"tostr(a5)"");\
  asm("movdqa xmm"tostr(b4)", xmm"tostr(a6)"");\
  asm("pxor xmm"tostr(a5)", xmm"tostr(a6)"");\
  asm("movdqa xmm"tostr(b5)", xmm"tostr(a7)"");\
  asm("pxor xmm"tostr(a6)", xmm"tostr(a7)"");\
  asm("pxor xmm"tostr(a7)", xmm"tostr(b6)"");\
  \
  /* build y4 y5 y6 ... in regs xmm8, xmm9, xmm10 by adding t_i*/\
  asm("pxor xmm"tostr(b0)", xmm"tostr(a4)"");\
  asm("pxor xmm"tostr(b6)", xmm"tostr(a4)"");\
  asm("pxor xmm"tostr(b1)", xmm"tostr(a5)"");\
  asm("pxor xmm"tostr(b7)", xmm"tostr(a5)"");\
  asm("pxor xmm"tostr(b2)", xmm"tostr(a6)"");\
  asm("pxor xmm"tostr(b0)", xmm"tostr(a6)"");\
  /* spill values y_4, y_5 to memory */\
  asm("movaps [rax+0*16], xmm"tostr(b0)"");   \
  asm("pxor xmm"tostr(b3)", xmm"tostr(a7)"");\
  asm("pxor xmm"tostr(b1)", xmm"tostr(a7)"");\
  asm("movaps [rax+1*16], xmm"tostr(b1)"");\
  asm("pxor xmm"tostr(b4)", xmm"tostr(a0)"");\
  asm("pxor xmm"tostr(b2)", xmm"tostr(a0)"");\
  /* save values t0, t1, t2 to xmm8, xmm9 and memory */\
  asm("movdqa xmm"tostr(b0)", xmm"tostr(a0)"");\
  asm("pxor xmm"tostr(b5)", xmm"tostr(a1)"");\
  asm("pxor xmm"tostr(b3)", xmm"tostr(a1)"");\
  asm("movdqa xmm"tostr(b1)", xmm"tostr(a1)"");\
  asm("pxor xmm"tostr(b6)", xmm"tostr(a2)"");\
  asm("pxor xmm"tostr(b4)", xmm"tostr(a2)"");\
  asm("movaps [rax+2*16], xmm"tostr(a2)"");\
  asm("pxor xmm"tostr(b7)", xmm"tostr(a3)"");\
  asm("pxor xmm"tostr(b5)", xmm"tostr(a3)"");\
  \
  /* compute x_i = t_i + t_{i+3} */\
  asm("pxor xmm"tostr(a0)", xmm"tostr(a3)"");\
  asm("pxor xmm"tostr(a1)", xmm"tostr(a4)"");\
  asm("pxor xmm"tostr(a2)", xmm"tostr(a5)"");\
  asm("pxor xmm"tostr(a3)", xmm"tostr(a6)"");\
  asm("pxor xmm"tostr(a4)", xmm"tostr(a7)"");\
  asm("pxor xmm"tostr(a5)", xmm"tostr(b0)"");\
  asm("pxor xmm"tostr(a6)", xmm"tostr(b1)"");\
  asm("pxor xmm"tostr(a7)", [rax+2*16]");\
  \
  /* compute z_i : double x_i using temp xmm8 and 1B xmm9 */\
  /* compute w_i : add y_{i+4} */\
  asm("movaps xmm"tostr(b1)", [rip+ALL_1B]");\
  MUL2(a0, b0, b1);\
  asm("pxor xmm"tostr(a0)", [rax+0*16]");\
  MUL2(a1, b0, b1);\
  asm("pxor xmm"tostr(a1)", [rax+1*16]");\
  MUL2(a2, b0, b1);\
  asm("pxor xmm"tostr(a2)", xmm"tostr(b2)"");\
  MUL2(a3, b0, b1);\
  asm("pxor xmm"tostr(a3)", xmm"tostr(b3)"");\
  MUL2(a4, b0, b1);\
  asm("pxor xmm"tostr(a4)", xmm"tostr(b4)"");\
  MUL2(a5, b0, b1);\
  asm("pxor xmm"tostr(a5)", xmm"tostr(b5)"");\
  MUL2(a6, b0, b1);\
  asm("pxor xmm"tostr(a6)", xmm"tostr(b6)"");\
  MUL2(a7, b0, b1);\
  asm("pxor xmm"tostr(a7)", xmm"tostr(b7)"");\
  \
  /* compute v_i : double w_i      */\
  /* add to y_4 y_5 .. v3, v4, ... */\
  MUL2(a0, b0, b1);\
  asm("pxor xmm"tostr(b5)", xmm"tostr(a0)"");\
  MUL2(a1, b0, b1);\
  asm("pxor xmm"tostr(b6)", xmm"tostr(a1)"");\
  MUL2(a2, b0, b1);\
  asm("pxor xmm"tostr(b7)", xmm"tostr(a2)"");\
  MUL2(a5, b0, b1);\
  asm("pxor xmm"tostr(b2)", xmm"tostr(a5)"");\
  MUL2(a6, b0, b1);\
  asm("pxor xmm"tostr(b3)", xmm"tostr(a6)"");\
  MUL2(a7, b0, b1);\
  asm("pxor xmm"tostr(b4)", xmm"tostr(a7)"");\
  MUL2(a3, b0, b1);\
  MUL2(a4, b0, b1);\
  asm("movaps xmm"tostr(b0)", [rax+0*16]");\
  asm("movaps xmm"tostr(b1)", [rax+1*16]");\
  asm("pxor xmm"tostr(b0)", xmm"tostr(a3)"");\
  asm("pxor xmm"tostr(b1)", xmm"tostr(a4)"");\
}/*MixBytes*/

#define Push_All_Regs() do{\
}while(0);

#define Pop_All_Regs() do{\
}while(0);

/* one round
 * i = round number
 * a0-a7 = input rows
 * b0-b7 = output rows
 */
#define ROUND(i, a0, a1, a2, a3, a4, a5, a6, a7, b0, b1, b2, b3, b4, b5, b6, b7){\
  /* AddRoundConstant */\
  asm ("movaps xmm"tostr(b1)", [rip+ROUND_CONST_Lx]");\
  asm ("pxor   xmm"tostr(a0)", [rip+ROUND_CONST_L0+"tostr(i)"*16]");\
  asm ("pxor   xmm"tostr(a1)", xmm"tostr(b1)"");\
  asm ("pxor   xmm"tostr(a2)", xmm"tostr(b1)"");\
  asm ("pxor   xmm"tostr(a3)", xmm"tostr(b1)"");\
  asm ("pxor   xmm"tostr(a4)", xmm"tostr(b1)"");\
  asm ("pxor   xmm"tostr(a5)", xmm"tostr(b1)"");\
  asm ("pxor   xmm"tostr(a6)", xmm"tostr(b1)"");\
  asm ("pxor   xmm"tostr(a7)", [rip+ROUND_CONST_L7+"tostr(i)"*16]");\
  /* ShiftBytes + SubBytes (interleaved) */\
  asm ("pxor xmm"tostr(b0)",  xmm"tostr(b0)"");\
  asm ("pshufb     xmm"tostr(a0)", [rip+SUBSH_MASK+0*16]");\
  asm ("aesenclast xmm"tostr(a0)", xmm"tostr(b0)"");\
  asm ("pshufb     xmm"tostr(a1)", [rip+SUBSH_MASK+1*16]");\
  asm ("aesenclast xmm"tostr(a1)", xmm"tostr(b0)"");\
  asm ("pshufb     xmm"tostr(a2)", [rip+SUBSH_MASK+2*16]");\
  asm ("aesenclast xmm"tostr(a2)", xmm"tostr(b0)"");\
  asm ("pshufb     xmm"tostr(a3)", [rip+SUBSH_MASK+3*16]");\
  asm ("aesenclast xmm"tostr(a3)", xmm"tostr(b0)"");\
  asm ("pshufb     xmm"tostr(a4)", [rip+SUBSH_MASK+4*16]");\
  asm ("aesenclast xmm"tostr(a4)", xmm"tostr(b0)"");\
  asm ("pshufb     xmm"tostr(a5)", [rip+SUBSH_MASK+5*16]");\
  asm ("aesenclast xmm"tostr(a5)", xmm"tostr(b0)"");\
  asm ("pshufb     xmm"tostr(a6)", [rip+SUBSH_MASK+6*16]");\
  asm ("aesenclast xmm"tostr(a6)", xmm"tostr(b0)"");\
  asm ("pshufb     xmm"tostr(a7)", [rip+SUBSH_MASK+7*16]");\
  asm ("aesenclast xmm"tostr(a7)", xmm"tostr(b0)"");\
  /* MixBytes */\
  MixBytes(a0, a1, a2, a3, a4, a5, a6, a7, b0, b1, b2, b3, b4, b5, b6, b7);\
}

/* 10 rounds, P and Q in parallel */
#define ROUNDS_P_Q(){\
  ROUND(0, 8, 9, 10, 11, 12, 13, 14, 15, 0, 1, 2, 3, 4, 5, 6, 7);\
  ROUND(1, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15);\
  ROUND(2, 8, 9, 10, 11, 12, 13, 14, 15, 0, 1, 2, 3, 4, 5, 6, 7);\
  ROUND(3, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15);\
  ROUND(4, 8, 9, 10, 11, 12, 13, 14, 15, 0, 1, 2, 3, 4, 5, 6, 7);\
  ROUND(5, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15);\
  ROUND(6, 8, 9, 10, 11, 12, 13, 14, 15, 0, 1, 2, 3, 4, 5, 6, 7);\
  ROUND(7, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15);\
  ROUND(8, 8, 9, 10, 11, 12, 13, 14, 15, 0, 1, 2, 3, 4, 5, 6, 7);\
  ROUND(9, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15);\
}

/* Matrix Transpose Step 1
 * input is a 512-bit state with two columns in one xmm
 * output is a 512-bit state with two rows in one xmm
 * inputs: i0-i3
 * outputs: i0, o1-o3
 * clobbers: t0
 */
#define Matrix_Transpose_A(i0, i1, i2, i3, o1, o2, o3, t0){\
  asm ("movaps xmm"tostr(t0)", [rip+TRANSP_MASK]");\
  \
  asm ("pshufb xmm"tostr(i0)", xmm"tostr(t0)"");\
  asm ("pshufb xmm"tostr(i1)", xmm"tostr(t0)"");\
  asm ("pshufb xmm"tostr(i2)", xmm"tostr(t0)"");\
  asm ("pshufb xmm"tostr(i3)", xmm"tostr(t0)"");\
  \
  asm ("movdqa xmm"tostr(o1)", xmm"tostr(i0)"");\
  asm ("movdqa xmm"tostr(t0)", xmm"tostr(i2)"");\
  \
  asm ("punpcklwd xmm"tostr(i0)", xmm"tostr(i1)"");\
  asm ("punpckhwd xmm"tostr(o1)", xmm"tostr(i1)"");\
  asm ("punpcklwd xmm"tostr(i2)", xmm"tostr(i3)"");\
  asm ("punpckhwd xmm"tostr(t0)", xmm"tostr(i3)"");\
  \
  asm ("pshufd xmm"tostr(i0)", xmm"tostr(i0)", 216");\
  asm ("pshufd xmm"tostr(o1)", xmm"tostr(o1)", 216");\
  asm ("pshufd xmm"tostr(i2)", xmm"tostr(i2)", 216");\
  asm ("pshufd xmm"tostr(t0)", xmm"tostr(t0)", 216");\
  \
  asm ("movdqa xmm"tostr(o2)", xmm"tostr(i0)"");\
  asm ("movdqa xmm"tostr(o3)", xmm"tostr(o1)"");\
  \
  asm ("punpckldq xmm"tostr(i0)", xmm"tostr(i2)"");\
  asm ("punpckldq xmm"tostr(o1)", xmm"tostr(t0)"");\
  asm ("punpckhdq xmm"tostr(o2)", xmm"tostr(i2)"");\
  asm ("punpckhdq xmm"tostr(o3)", xmm"tostr(t0)"");\
}/**/

/* Matrix Transpose Step 2
 * input are two 512-bit states with two rows in one xmm
 * output are two 512-bit states with one row of each state in one xmm
 * inputs: i0-i3 = P, i4-i7 = Q
 * outputs: (i0, o1-o7) = (P|Q)
 * possible reassignments: (output reg = input reg)
 * * i1 -> o3-7
 * * i2 -> o5-7
 * * i3 -> o7
 * * i4 -> o3-7
 * * i5 -> o6-7
 */
#define Matrix_Transpose_B(i0, i1, i2, i3, i4, i5, i6, i7, o1, o2, o3, o4, o5, o6, o7){\
  asm ("movdqa     xmm"tostr(o1)", xmm"tostr(i0)"");\
  asm ("movdqa     xmm"tostr(o2)", xmm"tostr(i1)"");\
  asm ("punpcklqdq xmm"tostr(i0)", xmm"tostr(i4)"");\
  asm ("punpckhqdq xmm"tostr(o1)", xmm"tostr(i4)"");\
  asm ("movdqa     xmm"tostr(o3)", xmm"tostr(i1)"");\
  asm ("movdqa     xmm"tostr(o4)", xmm"tostr(i2)"");\
  asm ("punpcklqdq xmm"tostr(o2)", xmm"tostr(i5)"");\
  asm ("punpckhqdq xmm"tostr(o3)", xmm"tostr(i5)"");\
  asm ("movdqa     xmm"tostr(o5)", xmm"tostr(i2)"");\
  asm ("movdqa     xmm"tostr(o6)", xmm"tostr(i3)"");\
  asm ("punpcklqdq xmm"tostr(o4)", xmm"tostr(i6)"");\
  asm ("punpckhqdq xmm"tostr(o5)", xmm"tostr(i6)"");\
  asm ("movdqa     xmm"tostr(o7)", xmm"tostr(i3)"");\
  asm ("punpcklqdq xmm"tostr(o6)", xmm"tostr(i7)"");\
  asm ("punpckhqdq xmm"tostr(o7)", xmm"tostr(i7)"");\
}/**/

/* Matrix Transpose Inverse Step 2
 * input are two 512-bit states with one row of each state in one xmm
 * output are two 512-bit states with two rows in one xmm
 * inputs: i0-i7 = (P|Q)
 * outputs: (i0, i2, i4, i6) = P, (o0-o3) = Q
 */
#define Matrix_Transpose_B_INV(i0, i1, i2, i3, i4, i5, i6, i7, o0, o1, o2, o3){\
  asm ("movdqa     xmm"tostr(o0)", xmm"tostr(i0)"");\
  asm ("punpcklqdq xmm"tostr(i0)", xmm"tostr(i1)"");\
  asm ("punpckhqdq xmm"tostr(o0)", xmm"tostr(i1)"");\
  asm ("movdqa     xmm"tostr(o1)", xmm"tostr(i2)"");\
  asm ("punpcklqdq xmm"tostr(i2)", xmm"tostr(i3)"");\
  asm ("punpckhqdq xmm"tostr(o1)", xmm"tostr(i3)"");\
  asm ("movdqa     xmm"tostr(o2)", xmm"tostr(i4)"");\
  asm ("punpcklqdq xmm"tostr(i4)", xmm"tostr(i5)"");\
  asm ("punpckhqdq xmm"tostr(o2)", xmm"tostr(i5)"");\
  asm ("movdqa     xmm"tostr(o3)", xmm"tostr(i6)"");\
  asm ("punpcklqdq xmm"tostr(i6)", xmm"tostr(i7)"");\
  asm ("punpckhqdq xmm"tostr(o3)", xmm"tostr(i7)"");\
}/**/

/* Matrix Transpose Output Step 2
 * input is one 512-bit state with two rows in one xmm
 * output is one 512-bit state with one row in the low 64-bits of one xmm
 * inputs: i0,i2,i4,i6 = S
 * outputs: (i0-7) = (0|S)
 */
#define Matrix_Transpose_O_B(i0, i1, i2, i3, i4, i5, i6, i7, t0){\
  asm ("pxor xmm"tostr(t0)", xmm"tostr(t0)"");\
  asm ("movdqa xmm"tostr(i1)", xmm"tostr(i0)"");\
  asm ("movdqa xmm"tostr(i3)", xmm"tostr(i2)"");\
  asm ("movdqa xmm"tostr(i5)", xmm"tostr(i4)"");\
  asm ("movdqa xmm"tostr(i7)", xmm"tostr(i6)"");\
  asm ("punpcklqdq xmm"tostr(i0)", xmm"tostr(t0)"");\
  asm ("punpckhqdq xmm"tostr(i1)", xmm"tostr(t0)"");\
  asm ("punpcklqdq xmm"tostr(i2)", xmm"tostr(t0)"");\
  asm ("punpckhqdq xmm"tostr(i3)", xmm"tostr(t0)"");\
  asm ("punpcklqdq xmm"tostr(i4)", xmm"tostr(t0)"");\
  asm ("punpckhqdq xmm"tostr(i5)", xmm"tostr(t0)"");\
  asm ("punpcklqdq xmm"tostr(i6)", xmm"tostr(t0)"");\
  asm ("punpckhqdq xmm"tostr(i7)", xmm"tostr(t0)"");\
}/**/

/* Matrix Transpose Output Inverse Step 2
 * input is one 512-bit state with one row in the low 64-bits of one xmm
 * output is one 512-bit state with two rows in one xmm
 * inputs: i0-i7 = (0|S)
 * outputs: (i0, i2, i4, i6) = S
 */
#define Matrix_Transpose_O_B_INV(i0, i1, i2, i3, i4, i5, i6, i7){\
  asm ("punpcklqdq xmm"tostr(i0)", xmm"tostr(i1)"");\
  asm ("punpcklqdq xmm"tostr(i2)", xmm"tostr(i3)"");\
  asm ("punpcklqdq xmm"tostr(i4)", xmm"tostr(i5)"");\
  asm ("punpcklqdq xmm"tostr(i6)", xmm"tostr(i7)"");\
}/**/


void INIT(uint64_t* h)
{
  /* __cdecl calling convention: */
  /* chaining value CV in rdi    */

  asm (".intel_syntax noprefix");
  asm volatile ("emms");

  /* load IV into registers xmm12 - xmm15 */
  asm ("movaps xmm12, [rdi+0*16]");
  asm ("movaps xmm13, [rdi+1*16]");
  asm ("movaps xmm14, [rdi+2*16]");
  asm ("movaps xmm15, [rdi+3*16]");

  /* transform chaining value from column ordering into row ordering */
  /* we put two rows (64 bit) of the IV into one 128-bit XMM register */
  Matrix_Transpose_A(12, 13, 14, 15, 2, 6, 7, 0);

  /* store transposed IV */
  asm ("movaps [rdi+0*16], xmm12");
  asm ("movaps [rdi+1*16], xmm2");
  asm ("movaps [rdi+2*16], xmm6");
  asm ("movaps [rdi+3*16], xmm7");

  asm volatile ("emms");
  asm (".att_syntax noprefix");
}

void TF512(uint64_t* h, uint64_t* m)
{
  /* __cdecl calling convention: */
  /* chaining value CV in rdi    */
  /* message M in rsi            */

  asm (".intel_syntax noprefix");
  Push_All_Regs();

  /* load message into registers xmm12 - xmm15 (Q = message) */
  asm ("movdqu xmm12, [rsi+0*16]");
  asm ("movdqu xmm13, [rsi+1*16]");
  asm ("movdqu xmm14, [rsi+2*16]");
  asm ("movdqu xmm15, [rsi+3*16]");

  /* transform message M from column ordering into row ordering */
  /* we first put two rows (2x64 bit) of the message into one 128-bit xmm register */
  Matrix_Transpose_A(12, 13, 14, 15, 2, 6, 7, 0);

  /* load previous chaining value */
  /* we first put two rows (64 bit) of the CV into one 128-bit xmm register */
  asm ("movaps xmm8, [rdi+0*16]");
  asm ("movaps xmm0, [rdi+1*16]");
  asm ("movaps xmm4, [rdi+2*16]");
  asm ("movaps xmm5, [rdi+3*16]");

  /* xor message to CV get input of P */
  /* result: CV+M in xmm8, xmm0, xmm4, xmm5 */
  asm ("pxor xmm8, xmm12");
  asm ("pxor xmm0, xmm2");
  asm ("pxor xmm4, xmm6");
  asm ("pxor xmm5, xmm7");

  /* there are now 2 rows of the Groestl state (P and Q) in each xmm register */
  /* unpack to get 1 row of P (64 bit) and Q (64 bit) into one xmm register */
  /* result: the 8 rows of P and Q in xmm8 - xmm12 */
  Matrix_Transpose_B(8, 0, 4, 5, 12, 2, 6, 7, 9, 10, 11, 12, 13, 14, 15);

  /* compute the two permutations P and Q in parallel */
  __attribute__ ((aligned (16))) unsigned char TEMP[3*16];
  asm (".att_syntax noprefix");
  asm (""::"a"(TEMP):);
  asm (".intel_syntax noprefix");
  ROUNDS_P_Q();

  /* unpack again to get two rows of P or two rows of Q in one xmm register */
  Matrix_Transpose_B_INV(8, 9, 10, 11, 12, 13, 14, 15, 0, 1, 2, 3);

  /* xor output of P and Q */
  /* result: P(CV+M)+Q(M) in xmm0...xmm3 */
  asm ("pxor xmm0, xmm8");
  asm ("pxor xmm1, xmm10");
  asm ("pxor xmm2, xmm12");
  asm ("pxor xmm3, xmm14");

  /* xor CV (feed-forward) */
  /* result: P(CV+M)+Q(M)+CV in xmm0...xmm3 */
  asm ("pxor xmm0, [rdi+0*16]");
  asm ("pxor xmm1, [rdi+1*16]");
  asm ("pxor xmm2, [rdi+2*16]");
  asm ("pxor xmm3, [rdi+3*16]");

  /* store CV */
  asm ("movaps [rdi+0*16], xmm0");
  asm ("movaps [rdi+1*16], xmm1");
  asm ("movaps [rdi+2*16], xmm2");
  asm ("movaps [rdi+3*16], xmm3");

  Pop_All_Regs();
  asm (".att_syntax noprefix");

  return;
}

void OF512(uint64_t* h)
{
  /* __cdecl calling convention: */
  /* chaining value CV in rdi    */

  asm (".intel_syntax noprefix");
  Push_All_Regs();

  /* load CV into registers xmm8, xmm10, xmm12, xmm14 */
  asm ("movaps xmm8,  [rdi+0*16]");
  asm ("movaps xmm10, [rdi+1*16]");
  asm ("movaps xmm12, [rdi+2*16]");
  asm ("movaps xmm14, [rdi+3*16]");

  /* there are now 2 rows of the CV in one xmm register */
  /* unpack to get 1 row of P (64 bit) into one half of an xmm register */
  /* result: the 8 input rows of P in xmm8 - xmm15 */
  Matrix_Transpose_O_B(8, 9, 10, 11, 12, 13, 14, 15, 0);

  /* compute the permutation P */
  /* result: the output of P(CV) in xmm8 - xmm15 */
  __attribute__ ((aligned (16))) unsigned char TEMP[3*16];
  asm (".att_syntax noprefix");
  asm (""::"a"(TEMP):);
  asm (".intel_syntax noprefix");
  ROUNDS_P_Q();

  /* unpack again to get two rows of P in one xmm register */
  /* result: P(CV) in xmm8, xmm10, xmm12, xmm14 */
  Matrix_Transpose_O_B_INV(8, 9, 10, 11, 12, 13, 14, 15);

  /* xor CV to P output (feed-forward) */
  /* result: P(CV)+CV in xmm8, xmm10, xmm12, xmm14 */
  asm ("pxor xmm8,  [rdi+0*16]");
  asm ("pxor xmm10, [rdi+1*16]");
  asm ("pxor xmm12, [rdi+2*16]");
  asm ("pxor xmm14, [rdi+3*16]");

  /* transform state back from row ordering into column ordering */
  /* result: final hash value in xmm9, xmm11 */
  Matrix_Transpose_A(8, 10, 12, 14, 4, 9, 11, 0);

  /* we only need to return the truncated half of the state */
  asm ("movaps [rdi+2*16], xmm9");
  asm ("movaps [rdi+3*16], xmm11");

  Pop_All_Regs();
  asm (".att_syntax noprefix");

  return;
}
/* hash.c     Aug 2011
 *
 * Groestl implementation for different versions.
 * Author: Krystian Matusiewicz, Günther A. Roland, Martin Schläffer
 *
 * This code is placed in the public domain
 */

/* digest up to len bytes of input (full blocks only) */
void Transform(hashState *ctx,
	       const uint8_t *in, 
	       unsigned long long len) {

    /* increment block counter */
    ctx->block_counter += len/SIZE;

    /* digest message, one block at a time */
    for (; len >= SIZE; len -= SIZE, in += SIZE)
      TF512((uint64_t*)ctx->chaining, (uint64_t*)in);

    asm volatile ("emms");
}

/* given state h, do h <- P(h)+h */
void OutputTransformation(hashState *ctx) {

    /* determine variant */
    OF512((uint64_t*)ctx->chaining);

    asm volatile ("emms");
}

/* initialise context */
void groestl_init(hashState* ctx) {
  ctx->chaining[COLS-1] = U64BIG((uint64_t)LENGTH);
  INIT(ctx->chaining);
}

/* update state with databitlen bits of input */
HashReturn groestl_update(hashState* ctx,
		  const BitSequence* input,
		  DataLength databitlen) {
  int index = 0;
  int msglen = (int)(databitlen/8);
  int rem = (int)(databitlen%8);

  /* non-integral number of message bytes can only be supplied in the
     last call to this function */
  if (ctx->bits_in_last_byte) return FAIL;

  /* if the buffer contains data that has not yet been digested, first
     add data to buffer until full */
  if (ctx->buf_ptr) {
    while (ctx->buf_ptr < SIZE && index < msglen) {
      ctx->buffer[(int)ctx->buf_ptr++] = input[index++];
    }
    if (ctx->buf_ptr < SIZE) {
      /* buffer still not full, return */
      if (rem) {
        ctx->bits_in_last_byte = rem;
        ctx->buffer[(int)ctx->buf_ptr++] = input[index];
      }
      return SUCCESS;
    }

    /* digest buffer */
    ctx->buf_ptr = 0;
    Transform(ctx, ctx->buffer, SIZE);
  }

  /* digest bulk of message */
  Transform(ctx, input+index, msglen-index);
  index += ((msglen-index)/SIZE)*SIZE;

  /* store remaining data in buffer */
  while (index < msglen) {
    ctx->buffer[(int)ctx->buf_ptr++] = input[index++];
  }

  /* if non-integral number of bytes have been supplied, store
     remaining bits in last byte, together with information about
     number of bits */
  if (rem) {
    ctx->bits_in_last_byte = rem;
    ctx->buffer[(int)ctx->buf_ptr++] = input[index];
  }
  return SUCCESS;
}

#define BILB ctx->bits_in_last_byte

/* finalise: process remaining data (including padding), perform
   output transformation, and write hash result to 'output' */
void groestl_final(hashState* ctx,
		 BitSequence* output) {
  int i, j = 0, hashbytelen = LENGTH/8;
  uint8_t *s = (BitSequence*)ctx->chaining;

  /* pad with '1'-bit and first few '0'-bits */
  if (BILB) {
    ctx->buffer[(int)ctx->buf_ptr-1] &= ((1<<BILB)-1)<<(8-BILB);
    ctx->buffer[(int)ctx->buf_ptr-1] ^= 0x1<<(7-BILB);
    BILB = 0;
  }
  else ctx->buffer[(int)ctx->buf_ptr++] = 0x80;

  /* pad with '0'-bits */
  if (ctx->buf_ptr > SIZE-LENGTHFIELDLEN) {
    /* padding requires two blocks */
    while (ctx->buf_ptr < SIZE) {
      ctx->buffer[(int)ctx->buf_ptr++] = 0;
    }
    /* digest first padding block */
    Transform(ctx, ctx->buffer, SIZE);
    ctx->buf_ptr = 0;
  }
  while (ctx->buf_ptr < SIZE-LENGTHFIELDLEN) {
    ctx->buffer[(int)ctx->buf_ptr++] = 0;
  }

  /* length padding */
  ctx->block_counter++;
  ctx->buf_ptr = SIZE;
  while (ctx->buf_ptr > SIZE-LENGTHFIELDLEN) {
    ctx->buffer[(int)--ctx->buf_ptr] = (uint8_t)ctx->block_counter;
    ctx->block_counter >>= 8;
  }

  /* digest final padding block */
  Transform(ctx, ctx->buffer, SIZE);
  /* perform output transformation */
  OutputTransformation(ctx);

  /* store hash result in output */
  for (i = SIZE-hashbytelen; i < SIZE; i++,j++) {
    output[j] = s[i];
  }

  /* zeroise relevant variables and deallocate memory */
  
  for (i = 0; i < COLS; i++) {
    ctx->chaining[i] = 0;
  }
  
  for (i = 0; i < SIZE; i++) {
    ctx->buffer[i] = 0;
  }
}
