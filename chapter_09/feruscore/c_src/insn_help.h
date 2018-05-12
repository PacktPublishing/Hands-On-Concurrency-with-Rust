#ifndef INSN_HELP_H
#define INSN_HELP_H

#define mBITS 3         /* number of bits for mode */
#define opBITS 5        /* bits for opcode */
#define moBITS 3        /* bits for modifier */
#define flBITS 2        /* bits for flags */

/* Positions of various fields
 */
#define maPOS 0
#define mbPOS (maPOS + mBITS)
#define moPOS (mbPOS + mBITS)
#define opPOS (moPOS + moBITS)
#define flPOS (opPOS + opBITS)

/* Various masks.  These extract a field once it has been
 * shifted to the least significant bits of the word.
 */
#define moMASK ((1<<moBITS)-1)
#define opMASK ((1<<opBITS)-1)
#define mMASK ((1<<mBITS)-1)
#define flMASK ((1<<flBITS)-1)
#define iMASK ( (1<<flPOS)-1 )

/*
 * Extract the flags of an instruction `in' field
 */
#define get_flags(in) ( ((in)>>flPOS) & flMASK )


/*
 * OP(o,m,ma,mb): This macro encodes an instruction `in' field
 *        from its various components (not flags).
 *
 *  o: opcode
 *  m: modifier
 *  ma: a-mode
 *  mb: b-mode
 *
 *  e.g. OP(SPL, mF, DIRECT, BPREDEC )
 *  is a
 *       spl.f $  , < 
 */
#define _OP(o,m) ( ((o)<<moBITS) | (m) )
#define OP( o, m, ma, mb ) ((_OP((o),(m))<<moPOS) | ((mb) << mbPOS) | (ma))


/*
 * flags
 */
enum {
    flB_START           /* start label */
};

#define fl_START (1<<flB_START)



/* Macros to take things mod CORESIZE
 *
 * Mod here is the `mathematical' modulo, with non-negative
 * results even with x negative.
 * 
 * MOD(x,M):    x mod CORESIZE
 * MODS(x,M):   x mod CORESIZE
 */
#define MODS(x,M) ( (int)(x)%(int)(M) >= 0          \
                    ? (int)(x)%(int)(M)             \
                    : (M) + ((int)(x)%(int)(M)) )
#define MOD(x,M) ( (x) % (M) )


#endif /* INSN_HELP_H */

