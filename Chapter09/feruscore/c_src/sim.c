/* This file is part of `exhaust', a memory array redcode simulator.
 * Copyright (C) 2002 M Joonas Pihlaja
 * Public Domain.
 */

/*
 * Thanks go to the pMARS authors and Ken Espiritu whose ideas have
 * been used in this simulator.  Especially Ken's effective addressing
 * calculation code in pMARS 0.8.6 has been adapted for use here.
 */

#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include "sim.h"
#include "insn.h"

/* Should we strip flags from instructions when loading?  By default,
   yes.  If so, then the simulator won't bother masking them off.  */
#ifndef SIM_STRIP_FLAGS
#define SIM_STRIP_FLAGS 1
#endif

/* DEBUG level:
 *     0: none
 *   >=1: disassemble each instruction (no output)
 *     2: print each instruction as it is executed
 */

/*
 * File scoped stuff
 */

#define DEF_MAX_WARS 2
#define DEF_CORESIZE 8000
#define DEF_PROCESSES 8000
#define DEF_CYCLES 80000

/* protos */
static int sim_proper(mars_t* mars, const uint16_t * const war_pos_tab, uint32_t *death_tab);

/*---------------------------------------------------------------
 * Simulator memory management
 */

void
sim_clear_core(mars_t* mars)
{
    memset(mars->coreMem, 0, mars->coresize*sizeof(insn_t));
}

/* NAME
 *     sim_alloc_bufs, sim_alloc_bufs2, sim_free_bufs --
 *              alloc and free buffers used in simulation
 *
 * SYNOPSIS
 *     insn_t *sim_alloc_bufs( unsigned int nwars, unsigned int coresize,
 *                             unsigned int processes, unsigned int cycles );
 *     insn_t *sim_alloc_bufs2( unsigned int nwars, unsigned int coresize,
 *              unsigned int processes, unsigned int cycles,
 *              unsigned int pspacesize );
 *     void sim_free_bufs();
 *
 * INPUTS
 *     nwar        -- number of warriors
 *     coresize    -- size of core
 *     processes   -- max no of processes / warrior
 *     cycles      -- the number of cycles to play before tie.
 *     pspacesize  -- size of p-space per warrior.  For sim_alloc_bufs(),
 *            it defaults to min(1,coresize/16).
 *
 * RESULTS
 *     These functions manage the core, queue, and w_t warrior info
 *     struct array memories.
 *
 *     Core_Mem, Queue_Mem, War_Tab and PSpace_mem memories are allocated
 *     or freed as requested.  Any earlier memories are freed before
 *     new allocations are made.
 *
 * RETURN VALUE
 *     sim_alloc_bufs(): the address of core memory, or NULL if
 *           out of memory.
 *     sim_free_bufs(): none
 *
 * GLOBALS
 *     All file scoped globals.
 */

void sim_free_bufs(mars_t* mars)
{
    free(mars->coreMem);
    free(mars->queueMem);
    free(mars->warTab);
}


/* allocate everything needed by simulator */
int sim_alloc_bufs(mars_t* mars) {
    mars->coreMem = (insn_t*)malloc(sizeof(insn_t) * mars->coresize);
    mars->queueMem = (insn_t**)malloc(sizeof(insn_t*)*(mars->nWarriors * mars->processes + 1));
    mars->warTab = (w_t*)malloc(sizeof(w_t)*mars->nWarriors);

    return (mars->coreMem
            && mars->queueMem
            && mars->warTab);
}


/* NAME
 *     sim_load_warrior -- load warrior code into core.
 *
 * SYNOPSIS
 *     sim_load_warrior(unsigned int pos, insn_t *code, unsigned int len);
 *
 * INPUTS
 *     pos -- The core address to load the warrior into.
 *     code -- array of instructions.
 *     len --
 *
 * DESCRIPTION
 *     This function is the preferred method to load warriors into core.
 *     It strips the instructions of any flags they may possess, and
 *     copies the instructions into core.
 *
 *     The code will happily overwrite any previous contents of core,
 *     so beware not to load warriors so that their code overlaps.
 *
 * NOTE
 *     The core must have been allocated previously with sim(), or
 *     preferably sim_alloc_bufs() and not freed since.
 *
 * RETURN VALUE
 *     0 -- warrior loaded OK.
 *    -1 -- core memory not allocated.
 *    -2 -- warrior length > core size.
 */
int
sim_load_warrior(mars_t* mars,
                 uint32_t pos,
                 const insn_t * const code,
                 uint16_t len)
{
    uint32_t i;
    uint16_t k;
    uint16_t in;
    uint32_t coresize = mars->coresize;
    insn_t* coreMem = mars->coreMem;

    for (i=0; i<len; i++) {
        k = (pos+i) % coresize;

#if SIM_STRIP_FLAGS
        in = code[i].in & iMASK;
#else
        in = code[i].in;
#endif

        coreMem[k].in = in;
        coreMem[k].a = code[i].a;
        coreMem[k].b = code[i].b;
    }
    return 0;
}

/*---------------------------------------------------------------
 * Simulator interface
 */

/* NAME
 *     sim, sim_mw -- public functions to simulate a round of Core War
 *
 * SYNOPSIS
 *     int sim_mw( unsigned int nwar, const uint16_t *war_pos_tab,
 *                 unsigned int *death_tab );
 *     int sim( int nwar, uint16_t w1_start, uint16_t w2_start,
 *      unsigned int cycles, void **ptr_result );
 *
 * INPUTS
 *     nwar        -- number of warriors
 *     w1_start, w2_start -- core addresses of first processes
 *                    warrior 1 and warrior 2. Warrior 1 executes first.
 *     cycles      -- the number of cycles to play before tie.
 *     ptr_result  -- NULL, except when requesting the address of core.
 *     war_pos_tab -- core addresses where warriors are loaded in
 *            the order they are to be executed.
 *     death_tab   -- the table where dead warrior indices are stored
 *
 * DESCRIPTION
 *     The real simulator is inside sim_proper() to which sim() and
 *     sim_mw() are proxies.  sim_mw() reads the warrior position
 *     of the ith warrior from war_tab_pos[i-1].
 *
 * RESULTS
 *     The warriors fight their fight in core which gets messed up in
 *     the process.  If a warrior died during the fight then its p-space
 *     location 0 is cleared.  Otherwise the number of warriors alive
 *     at the end of the battle is stored into its p-space location 0.
 *
 *     sim_mw() stores indices of warriors that die into the death_tab
 *     array in the order of death.  Warrior indices start from 0.
 *
 *     For sim(): If nwar == -1 then buffers of default size for
 *     max. two warriors are allocated and the address of the core
 *     memory is returned via the ptr_result pointer.
 *
 * RETURN VALUE
 *     sim_mw(): the number of warriors still alive at the end of the
 *       battle.
 *       -1: simulator panic attack -- something's gone wrong
 *
 *     sim():
 *       single warrior: 0: warrior suicided, 1: warrior didn't die.
 *       one-on-one two warriors:
 *        0: warrior 1 won, 1: warrior 2 won, 2: tie
 *       -1: simulator panic attack -- something's gone wrong
 *
 * GLOBALS
 *     All file scoped globals */

int
sim_mw(mars_t* mars, const uint16_t *const war_pos_tab, uint32_t* death_tab)
{
    int alive_count;
    alive_count = sim_proper(mars, war_pos_tab, death_tab);
    return alive_count;
}

/*-------------------------------------------------------------------------
 * private functions
 */

/* NAME
 *     sim_proper -- the real simulator code
 *
 * SYNOPSIS
 *     int sim_proper( unsigned int nwar,
 *                     const uint16_t *war_pos_tab,
 *                     unsigned int *death_tab );
 *
 * INPUTS
 *     nwar        -- number of warriors
 *     war_pos_tab -- core addresses where warriors are loaded in
 *            the order they are to be executed.
 *     death_tab   -- the table where dead warrior indices are stored
 *
 * RESULTS
 *     The warriors fight their fight in core which gets messed up in
 *     the process.  The indices of warriors that die are stored into
 *     the death_tab[] array in the order of death.  Warrior indices
 *     start from 0.
 *
 * RETURN VALUE
 *     The number of warriors still alive at the end of the
 *     battle or -1 on an anomalous condition.
 *
 * GLOBALS
 *     All file scoped globals
 */

/* Various macros:
 *
 *  queue(x): Inserts a core address 'x' to the head of the current
 *            warrior's process queue.  Assumes the warrior's
 *        tail pointer is inside the queue buffer.
 *
 * x, y must be in 0..coresize-1 for the following macros:
 *
 * INCMOD(x): x = x+1 mod coresize
 * DECMOD(x): x = x-1 mod coresize
 * ADDMOD(z,x,y): z = x+y mod coresize
 * SUBMOD(z,x,y): z = x-y mod coresize
 */

#define queue(x)                                                        \
    do {                                                                \
        insn_t** w_tail = w->tail;                                      \
        *(w_tail) = (x); if (++(w_tail) == queue_end) w_tail = queue_start; \
        w->tail = w_tail;                                               \
    } while(0)

#define INCMOD(x) do { if ( ++(x) == coresize ) (x) = 0; } while (0)
#define IPINCMOD(x) do { if ( ++(x) == CoreEnd ) (x) = core; } while (0)
#define DECMOD(x) do { if ((x) == 0) (x)=coresize1; else --(x); } while (0)
#define IPDECMOD(x) do { if ((x)==0) (x)=CoreEnd1; else --(x); } while (0)
#define ADDMOD(z,x,y) do { (z) = (x)+(y); if ((z)>=coresize) (z) -= coresize; } while (0)
/* z is unsigned! overflow occurs. */
#define SUBMOD(z,x,y) do { (z) = (x)-(y); if ((z)>=coresize) (z) += coresize; } while (0)

int
sim_proper(mars_t* mars, const uint16_t * const war_pos_tab, uint32_t* death_tab )
{
    /*
     * Core and Process queue memories.
     *
     * The warriors share a common cyclic buffer for use as a process
     * queue which the contains core addresses where active processes
     * are.  The buffer has size N*P+1, where N = number of warriors,
     * P = maximum number of processes / warrior.
     *
     * Each warrior has a fixed slice of the buffer for its own process
     * queue which are initially allocated to the warriors in reverse
     * order. i.e. if the are N warriors w1, w2, ..., wN, the slice for
     * wN is 0..P-1, w{N-1} has P..2P-1, until w1 has (N-1)P..NP-1.
     *
     * The core address of the instruction is fetched from the head of
     * the process queue and processes are pushed to the tail, so the
     * individual slices slide along at one location per executed
     * instruction.  The extra '+1' in the buffer size is to have free
     * space to slide the slices along.
     *
     * For two warriors w1, w2:
     *
     * |\......../|\......../| |
     * | w2 queue | w1 queue | |
     * 0          P         2P 2P+1
     */

    /*
     * Cache Registers.
     *
     * The '94 draft specifies that the redcode processor model be
     * 'in-register'.  That is, the current instruction and the
     * instructions at the effective addresses (ea's) be cached in
     * registers during instruction execution, rather than have
     * core memory accessed directly when the operands are needed.  This
     * causes differences from the 'in-memory' model.  e.g. MOV 0,>0
     * doesn't change the instruction's b-field since the instruction at
     * the a-field's effective address (i.e. the instruction itself) was
     * cached before the post-increment happened.
     *
     * There are conceptually three registers: IN, A, and B.  IN is the
     * current instruction, and A, B are the ones at the a- and
     * b-fields' effective addresses respectively.
     *
     * We don't actually cache the complete instructions, but rather
     * only the *values* of their a- and b-field.  This is because
     * currently there is no way effective address computations can
     * modify the opcode, modifier, or addressing modes of an
     * instruction.
     */


    /*
     * misc.
     */
    uint32_t processes = mars->processes;
    insn_t* const core = mars->coreMem;
    insn_t** const queue_start = mars->queueMem;
    uint32_t nwar = mars->nWarriors;
    insn_t** const queue_end = mars->queueMem + nwar * mars->processes + 1;
    w_t* w;         /* current warrior */
    const unsigned int coresize = mars->coresize;
    const unsigned int coresize1 = coresize-1; /* size of core, size of core - 1 */
    insn_t* const CoreEnd = core + coresize; /* point after last instruction */
    uint32_t cycles = nwar * mars->cycles; /* set instruction executions until tie counter */
    int alive_cnt = nwar;
    uint32_t max_alive_proc = nwar * mars->processes;
    insn_t **pofs = queue_end-1;

#if DEBUG >= 1
    insn_t insn;            /* used for disassembly */
    char debug_line[256];       /* ditto */
#endif
    w_t* warTab = mars->warTab;

    warTab[0].succ = &(warTab[nwar-1]);
    warTab[nwar-1].pred = &(warTab[0]);
    {
        uint32_t ftmp = 0;     /* temps */

        do {
            int t = nwar-1-ftmp;
            if ( t > 0 ) warTab[t].succ = &(warTab[t-1]);
            if ( t < (int)nwar-1 ) warTab[t].pred = &(warTab[t+1]);
            pofs -= processes;
            *pofs = &(core[war_pos_tab[ftmp]]);
            warTab[t].head = pofs;
            warTab[t].tail = pofs+1;
            warTab[t].nprocs = 1;
            warTab[t].id = ftmp;
            ftmp++;
        } while ( ftmp < nwar );
    }

    w = &(warTab[nwar-1]);

    /*******************************************************************
     * Main loop - optimize here
     */
    do {
        /* 'in' field of current insn for decoding */
        uint32_t in;

        /* A register values */
        uint16_t ra_a, ra_b;

        /* B register values */
        uint16_t rb_a, rb_b;

        insn_t *pta;
        insn_t *ptb;
        unsigned int mode;

        insn_t* ip = *(w->head);
        if ( ++(w->head) == queue_end ) w->head = queue_start;
        in = ip->in;        /* note: flags must be unset! */
#if !SIM_STRIP_FLAGS
        in = in & iMASK;        /* strip flags. */
#endif
        rb_a = ra_a = ip->a;
        rb_b = ip->b;

#if DEBUG >= 1
        insn = *ip;
        dis1( debug_line, insn, coresize);
#endif

        mode = in & mMASK;

        /* a-mode calculation */
        if (mode == EX_IMMEDIATE) {
            /*printf("IMMEDIATE\n");*/
            ra_b = rb_b;
            pta = ip;
        } else if (mode == EX_DIRECT) {
            /*printf("DIRECT\n");*/
            pta = ip + ra_a; if (pta >= CoreEnd) pta -= coresize;
            ra_a = pta->a;
            ra_b = pta->b;
        } else if (mode == EX_BINDIRECT) {
            /*printf("BINDIRECT\n");*/
            pta = ip + ra_a; if (pta >= CoreEnd) pta -= coresize;
            pta = pta + pta->b; if (pta >= CoreEnd) pta -= coresize;
            ra_a = pta->a;      /* read in registers */
            ra_b = pta->b;
        } else if (mode == EX_APOSTINC) {
            /*printf("APOSTINC\n");*/
            pta = ip + ra_a; if (pta >= CoreEnd) pta -= coresize;
            {uint16_t* f = &(pta->a);
                pta = pta + pta->a; if (pta >= CoreEnd) pta -= coresize;
                ra_a = pta->a;      /* read in registers */
                ra_b = pta->b;
                INCMOD(*f);}
        } else if (mode == EX_BPOSTINC) {
            /*printf("BPOSTINC\n");*/
            pta = ip + ra_a; if (pta >= CoreEnd) pta -= coresize;
            {uint16_t* f = &(pta->b);
                pta = pta + pta->b; if (pta >= CoreEnd) pta -= coresize;
                ra_a = pta->a;      /* read in registers */
                ra_b = pta->b;
                INCMOD(*f);}
        } else if (mode == EX_APREDEC) {
            /*printf("APREDEC\n");*/
            pta = ip + ra_a; if (pta >= CoreEnd) pta -= coresize;
            DECMOD(pta->a);
            pta = pta + pta->a; if (pta >= CoreEnd) pta -= coresize;
            ra_a = pta->a;      /* read in registers */
            ra_b = pta->b;
        } else if (mode == EX_BPREDEC) {
            /*printf("BPREDEC\n");*/
            pta = ip + ra_a; if (pta >= CoreEnd) pta -= coresize;
            DECMOD(pta->b);
            pta = pta + pta->b; if (pta >= CoreEnd) pta -= coresize;
            ra_a = pta->a;      /* read in registers */
            ra_b = pta->b;
        } else { /* AINDIRECT */
            /*printf("AINDIRECT\n");*/
            pta = ip + ra_a; if (pta >= CoreEnd) pta -= coresize;
            pta = pta + pta->a; if (pta >= CoreEnd) pta -= coresize;
            ra_a = pta->a;      /* read in registers */
            ra_b = pta->b;
        }

        mode = in & (mMASK<<mBITS);

        /* special mov.i code to improve performance */
        if ((in & 16320) == (_OP(EX_MOV, EX_mI) << (mBITS*2))) {
            if (mode == EX_DIRECT<<mBITS) {
                /*++modes[1];*/
                /* 150886214*/  ptb = ip + rb_b; if (ptb >= CoreEnd) ptb -= coresize;
            } else if (mode == EX_BPOSTINC<<mBITS) {
                /*++modes[5];*/
                ptb = ip + rb_b; if (ptb >= CoreEnd) ptb -= coresize;
                {uint16_t* f = &(ptb->b);
                    ptb = ptb + *f; if (ptb >= CoreEnd) ptb -= coresize;
                    /*  92075270*/  INCMOD(*f);}
            } else if (mode == EX_AINDIRECT<<mBITS) {
                /*++modes[7];*/
                ptb = ip + rb_b; if (ptb >= CoreEnd) ptb -= coresize;
                /*  39436060*/  ptb = ptb + ptb->a; if (ptb >= CoreEnd) ptb -= coresize;
            } else if (mode == EX_APOSTINC<<mBITS) {
                /*++modes[2];*/
                ptb = ip + rb_b; if (ptb >= CoreEnd) ptb -= coresize;
                {uint16_t* f = &(ptb->a);
                    ptb = ptb + *f; if (ptb >= CoreEnd) ptb -= coresize;
                    /*  32635122*/  INCMOD(*f);}
            } else if (mode == EX_APREDEC<<mBITS) {
                /*++modes[0];*/
                ptb = ip + rb_b; if (ptb >= CoreEnd) ptb -= coresize;
                DECMOD(ptb->a);
                /*  19211424*/  ptb = ptb + ptb->a; if (ptb >= CoreEnd) ptb -= coresize;
            } else if (mode == EX_BPREDEC<<mBITS) {
                /*++modes[3];*/
                ptb = ip + rb_b; if (ptb >= CoreEnd) ptb -= coresize;
                DECMOD(ptb->b);
                /*  11269800*/  ptb = ptb + ptb->b; if (ptb >= CoreEnd) ptb -= coresize;
            } else if (mode == EX_BINDIRECT<<mBITS) {
                /*++modes[6];*/
                ptb = ip + rb_b; if (ptb >= CoreEnd) ptb -= coresize;
                /*  8582998*/   ptb = ptb + ptb->b; if (ptb >= CoreEnd) ptb -= coresize;
            } else { /* EX_IMMEDIATE */
                /*++modes[4];*/
                /*      1446*/  ptb = ip;
            }
            ptb->a = ra_a;
            ptb->b = ra_b;
            ptb->in = pta->in;
            IPINCMOD(ip);
            queue(ip);
            goto noqueue;
        }


        /*15360:
         *              0  0  1  1  1  1  0  0  0  0  0  0  0  0  0  0
         * bit         15 14 13 12 11 10  9  8  7  6  5  4  3  2  1  0
         * field   | flags | |- op-code  -| |-.mod-| |b-mode| |a-mode|
         */
        if (!(in & 15360)) {
            /* DAT or SPL */
            if (mode == EX_IMMEDIATE<<mBITS) {
            } else if (mode == EX_DIRECT<<mBITS) {
            } else if (mode == EX_BPOSTINC<<mBITS) {
                ptb = ip + rb_b; if (ptb >= CoreEnd) ptb -= coresize;
                INCMOD(ptb->b);
            } else if (mode == EX_BPREDEC<<mBITS) {
                ptb = ip + rb_b; if (ptb >= CoreEnd) ptb -= coresize;
                DECMOD(ptb->b);
            } else if (mode == EX_APREDEC<<mBITS) {
                ptb = ip + rb_b; if (ptb >= CoreEnd) ptb -= coresize;
                DECMOD(ptb->a);
            } else if (mode == EX_APOSTINC<<mBITS) {
                ptb = ip + rb_b; if (ptb >= CoreEnd) ptb -= coresize;
                INCMOD(ptb->a);
            } /* BINDIRECT, AINDIRECT */

            if (in & 512) {
                /* SPL */
                IPINCMOD(ip);
                queue(ip);
                if ( w->nprocs < processes ) {
                    ++w->nprocs;
                    queue(pta);
                }
                /* in the endgame, check if a tie is inevitable */
                if (cycles < max_alive_proc) {
                    w_t* w_iterator = w->succ;

                    /* break if all warriors have more processes than cycles */
                    while ((w_iterator->nprocs * alive_cnt > cycles) && (w_iterator != w)) w_iterator = w_iterator->succ;
                    if (w_iterator->nprocs*alive_cnt  > cycles) {
                        /*printf("stopping at %d\n", cycles);*/
                        goto out;
                    }
                }
            }
            else {
                /* DAT */
            die:
                if (--w->nprocs) goto noqueue;
                w->pred->succ = w->succ;
                w->succ->pred = w->pred;
                *death_tab++ = w->id;
                cycles = cycles - cycles/alive_cnt; /* nC+k -> (n-1)C+k */
                max_alive_proc = alive_cnt * processes;
                if ( --alive_cnt <= 1 )
                    goto out;
            }
            goto noqueue;
        }


        /* b-mode calculation */
        if (mode == EX_APREDEC<<mBITS) {
            /*printf("APREDEC\n");*/
            ptb = ip + rb_b; if (ptb >= CoreEnd) ptb -= coresize;
            DECMOD(ptb->a);
            ptb = ptb + ptb->a; if (ptb >= CoreEnd) ptb -= coresize;
            rb_a = ptb->a;      /* read in registers */
            rb_b = ptb->b;
        } else if (mode == EX_DIRECT<<mBITS) {
            /*printf("DIRECT\n");*/
            ptb = ip + rb_b; if (ptb >= CoreEnd) ptb -= coresize;
            rb_a = ptb->a;
            rb_b = ptb->b;
        } else if (mode == EX_APOSTINC<<mBITS) {
            /*printf("APOSTINC\n");*/
            ptb = ip + rb_b; if (ptb >= CoreEnd) ptb -= coresize;
            {uint16_t* f = &(ptb->a);
                ptb = ptb + ptb->a; if (ptb >= CoreEnd) ptb -= coresize;
                rb_a = ptb->a;      /* read in registers */
                rb_b = ptb->b;
                INCMOD(*f);}
        } else if (mode == EX_BPREDEC<<mBITS) {
            /*printf("BPREDEC\n");*/
            ptb = ip + rb_b; if (ptb >= CoreEnd) ptb -= coresize;
            DECMOD(ptb->b);
            ptb = ptb + ptb->b; if (ptb >= CoreEnd) ptb -= coresize;
            rb_a = ptb->a;      /* read in registers */
            rb_b = ptb->b;
        } else if (mode == EX_IMMEDIATE<<mBITS) {
            /*printf("IMMEDIATE\n");*/
            ptb = ip;
        } else if (mode == EX_BPOSTINC<<mBITS) {
            /*printf("BPOSTINC\n");*/
            ptb = ip + rb_b; if (ptb >= CoreEnd) ptb -= coresize;
            {uint16_t* f = &(ptb->b);
                ptb = ptb + ptb->b; if (ptb >= CoreEnd) ptb -= coresize;
                rb_a = ptb->a;      /* read in registers */
                rb_b = ptb->b;
                INCMOD(*f);}
        } else if (mode == EX_BINDIRECT<<mBITS) {
            /*printf("BINDIRECT\n");*/
            ptb = ip + rb_b; if (ptb >= CoreEnd) ptb -= coresize;
            ptb = ptb + ptb->b; if (ptb >= CoreEnd) ptb -= coresize;
            rb_a = ptb->a;      /* read in registers */
            rb_b = ptb->b;
        } else { /* AINDIRECT */
            /*printf("AINDIRECT\n");*/
            ptb = ip + rb_b; if (ptb >= CoreEnd) ptb -= coresize;
            ptb = ptb + ptb->a; if (ptb >= CoreEnd) ptb -= coresize;
            rb_a = ptb->a;      /* read in registers */
            rb_b = ptb->b;
        }

#if DEBUG == 2
        /* Debug output */
        printf("%6d %4ld  %s  |%4ld, d %4ld,%4ld a %4ld,%4ld b %4ld,%4ld\n",
               cycles, ip-core, debug_line,
               w->nprocs, pta-core, ptb-core,
               ra_a, ra_b, rb_a, rb_b );
#endif

        /*
         * Execute the instruction on opcode.modifier
         */


        switch ( in>>(mBITS*2) ) {

        case _OP(EX_MOV, EX_mA):
            ptb->a = ra_a;
            break;
        case _OP(EX_MOV, EX_mF):
            ptb->a = ra_a;
        case _OP(EX_MOV, EX_mB):
            ptb->b = ra_b;
            break;
        case _OP(EX_MOV, EX_mAB):
            ptb->b = ra_a;
            break;
        case _OP(EX_MOV, EX_mX):
            ptb->b = ra_a;
        case _OP(EX_MOV, EX_mBA):
            ptb->a = ra_b;
            break;

        case _OP(EX_MOV, EX_mI):
            printf("unreachable code reached. You have a problem!\n");
            break;

        case _OP(EX_DJN,EX_mBA):
        case _OP(EX_DJN,EX_mA):
            DECMOD(ptb->a);
            if ( rb_a == 1 ) break;
            queue(pta);
            goto noqueue;

        case _OP(EX_DJN,EX_mAB):
        case _OP(EX_DJN,EX_mB):
            DECMOD(ptb->b);
            if ( rb_b == 1 ) break;
            queue(pta);
            goto noqueue;

        case _OP(EX_DJN,EX_mX):
        case _OP(EX_DJN,EX_mI):
        case _OP(EX_DJN,EX_mF):
            DECMOD(ptb->a);
            DECMOD(ptb->b);
            /* if ( rb_a == 1 && rb_b == 1 ) break; */
            if ( rb_a == 1 && rb_b == 1 ) break;
            queue(pta);
            goto noqueue;


        case _OP(EX_ADD, EX_mI):
        case _OP(EX_ADD, EX_mF):
            ADDMOD(ptb->b, ra_b, rb_b );
        case _OP(EX_ADD, EX_mA):
            ADDMOD(ptb->a, ra_a, rb_a );
            break;
        case _OP(EX_ADD, EX_mB):
            ADDMOD(ptb->b, ra_b, rb_b );
            break;
        case _OP(EX_ADD, EX_mX):
            ADDMOD(ptb->a, ra_b, rb_a );
        case _OP(EX_ADD, EX_mAB):
            ADDMOD(ptb->b, ra_a, rb_b );
            break;
        case _OP(EX_ADD, EX_mBA):
            ADDMOD(ptb->a, ra_b, rb_a );
            break;


        case _OP(EX_JMZ, EX_mBA):
        case _OP(EX_JMZ, EX_mA):
            if ( rb_a )
                break;
            queue(pta);
            goto noqueue;

        case _OP(EX_JMZ, EX_mAB):
        case _OP(EX_JMZ, EX_mB):
            if ( rb_b )
                break;
            queue(pta);
            goto noqueue;

        case _OP(EX_JMZ, EX_mX):
        case _OP(EX_JMZ, EX_mF):
        case _OP(EX_JMZ, EX_mI):
            if ( rb_a || rb_b )
                break;
            queue(pta);
            goto noqueue;


        case _OP(EX_SUB, EX_mI):
        case _OP(EX_SUB, EX_mF):
            SUBMOD(ptb->b, rb_b, ra_b );
        case _OP(EX_SUB, EX_mA):
            SUBMOD(ptb->a, rb_a, ra_a);
            break;
        case _OP(EX_SUB, EX_mB):
            SUBMOD(ptb->b, rb_b, ra_b );
            break;
        case _OP(EX_SUB, EX_mX):
            SUBMOD(ptb->a, rb_a, ra_b );
        case _OP(EX_SUB, EX_mAB):
            SUBMOD(ptb->b, rb_b, ra_a );
            break;
        case _OP(EX_SUB, EX_mBA):
            SUBMOD(ptb->a, rb_a, ra_b );
            break;


        case _OP(EX_SEQ, EX_mA):
            if ( ra_a == rb_a )
                IPINCMOD(ip);
            break;
        case _OP(EX_SEQ, EX_mB):
            if ( ra_b == rb_b )
                IPINCMOD(ip);
            break;
        case _OP(EX_SEQ, EX_mAB):
            if ( ra_a == rb_b )
                IPINCMOD(ip);
            break;
        case _OP(EX_SEQ, EX_mBA):
            if ( ra_b == rb_a )
                IPINCMOD(ip);
            break;

        case _OP(EX_SEQ, EX_mI):
            if ( pta->in != ptb->in )
                break;
        case _OP(EX_SEQ, EX_mF):
            if ( ra_a == rb_a && ra_b == rb_b )
                IPINCMOD(ip);
            break;
        case _OP(EX_SEQ, EX_mX):
            if ( ra_a == rb_b && ra_b == rb_a )
                IPINCMOD(ip);
            break;


        case _OP(EX_SNE, EX_mA):
            if ( ra_a != rb_a )
                IPINCMOD(ip);
            break;
        case _OP(EX_SNE, EX_mB):
            if ( ra_b != rb_b )
                IPINCMOD(ip);
            break;
        case _OP(EX_SNE, EX_mAB):
            if ( ra_a != rb_b )
                IPINCMOD(ip);
            break;
        case _OP(EX_SNE, EX_mBA):
            if ( ra_b != rb_a )
                IPINCMOD(ip);
            break;

        case _OP(EX_SNE, EX_mI):
            if ( pta->in != ptb->in ) {
                IPINCMOD(ip);
                break;
            }
            /* fall through */
        case _OP(EX_SNE, EX_mF):
            if ( ra_a != rb_a || ra_b != rb_b )
                IPINCMOD(ip);
            break;
        case _OP(EX_SNE, EX_mX):
            if ( ra_a != rb_b || ra_b != rb_a )
                IPINCMOD(ip);
            break;


        case _OP(EX_JMN, EX_mBA):
        case _OP(EX_JMN, EX_mA):
            if (! rb_a )
                break;
            queue(pta);
            goto noqueue;

        case _OP(EX_JMN, EX_mAB):
        case _OP(EX_JMN, EX_mB):
            if (! rb_b )
                break;
            queue(pta);
            goto noqueue;

        case _OP(EX_JMN, EX_mX):
        case _OP(EX_JMN, EX_mF):
        case _OP(EX_JMN, EX_mI):
            if (rb_a || rb_b) {
                queue(pta);
                goto noqueue;
            }
            break;


        case _OP(EX_JMP, EX_mA):
        case _OP(EX_JMP, EX_mB):
        case _OP(EX_JMP, EX_mAB):
        case _OP(EX_JMP, EX_mBA):
        case _OP(EX_JMP, EX_mX):
        case _OP(EX_JMP, EX_mF):
        case _OP(EX_JMP, EX_mI):
            queue(pta);
            goto noqueue;



        case _OP(EX_SLT, EX_mA):
            if (ra_a < rb_a)
                IPINCMOD(ip);
            break;
        case _OP(EX_SLT, EX_mAB):
            if (ra_a < rb_b)
                IPINCMOD(ip);
            break;
        case _OP(EX_SLT, EX_mB):
            if (ra_b < rb_b)
                IPINCMOD(ip);
            break;
        case _OP(EX_SLT, EX_mBA):
            if (ra_b < rb_a)
                IPINCMOD(ip);
            break;
        case _OP(EX_SLT, EX_mI):
        case _OP(EX_SLT, EX_mF):
            if (ra_a < rb_a && ra_b < rb_b)
                IPINCMOD(ip);
            break;
        case _OP(EX_SLT, EX_mX):
            if (ra_a < rb_b && ra_b < rb_a)
                IPINCMOD(ip);
            break;


        case _OP(EX_MODM, EX_mI):
        case _OP(EX_MODM, EX_mF):
            if ( ra_a ) ptb->a = rb_a % ra_a;
            if ( ra_b ) ptb->b = rb_b % ra_b;
            if (!ra_a || !ra_b) goto die;
            break;
        case _OP(EX_MODM, EX_mX):
            if ( ra_b ) ptb->a = rb_a % ra_b;
            if ( ra_a ) ptb->b = rb_b % ra_a;
            if (!ra_b || !ra_a) goto die;
            break;
        case _OP(EX_MODM, EX_mA):
            if ( !ra_a ) goto die;
            ptb->a = rb_a % ra_a;
            break;
        case _OP(EX_MODM, EX_mB):
            if ( !ra_b ) goto die;
            ptb->b = rb_b % ra_b;
            break;
        case _OP(EX_MODM, EX_mAB):
            if ( !ra_a ) goto die;
            ptb->b = rb_b % ra_a;
            break;
        case _OP(EX_MODM, EX_mBA):
            if ( !ra_b ) goto die;
            ptb->a = rb_a % ra_b;
            break;


        case _OP(EX_MUL, EX_mI):
        case _OP(EX_MUL, EX_mF):
            ptb->b = (rb_b * ra_b) % coresize;
        case _OP(EX_MUL, EX_mA):
            ptb->a = (rb_a * ra_a) % coresize;
            break;
        case _OP(EX_MUL, EX_mB):
            ptb->b = (rb_b * ra_b) % coresize;
            break;
        case _OP(EX_MUL, EX_mX):
            ptb->a = (rb_a * ra_b) % coresize;
        case _OP(EX_MUL, EX_mAB):
            ptb->b = (rb_b * ra_a) % coresize;
            break;
        case _OP(EX_MUL, EX_mBA):
            ptb->a = (rb_a * ra_b) % coresize;
            break;


        case _OP(EX_DIV, EX_mI):
        case _OP(EX_DIV, EX_mF):
            if ( ra_a ) ptb->a = rb_a / ra_a;
            if ( ra_b ) ptb->b = rb_b / ra_b;
            if (!ra_a || !ra_b) goto die;
            break;
        case _OP(EX_DIV, EX_mX):
            if ( ra_b ) ptb->a = rb_a / ra_b;
            if ( ra_a ) ptb->b = rb_b / ra_a;
            if (!ra_b || !ra_a) goto die;
            break;
        case _OP(EX_DIV, EX_mA):
            if ( !ra_a ) goto die;
            ptb->a = rb_a / ra_a;
            break;
        case _OP(EX_DIV, EX_mB):
            if ( !ra_b ) goto die;
            ptb->b = rb_b / ra_b;
            break;
        case _OP(EX_DIV, EX_mAB):
            if ( !ra_a ) goto die;
            ptb->b = rb_b / ra_a;
            break;
        case _OP(EX_DIV, EX_mBA):
            if ( !ra_b ) goto die;
            ptb->a = rb_a / ra_b;
            break;


        case _OP(EX_NOP,EX_mI):
        case _OP(EX_NOP,EX_mX):
        case _OP(EX_NOP,EX_mF):
        case _OP(EX_NOP,EX_mA):
        case _OP(EX_NOP,EX_mAB):
        case _OP(EX_NOP,EX_mB):
        case _OP(EX_NOP,EX_mBA):
            break;

#if DEBUG > 0
        default:
            alive_cnt = -1;
            goto out;
#endif
        }

        IPINCMOD(ip);
        queue(ip);
    noqueue:
        w = w->succ;
    } while(--cycles);

out:
#if DEBUG == 1
    printf("cycles: %d\n", cycles);
#endif
    return alive_cnt;
}
