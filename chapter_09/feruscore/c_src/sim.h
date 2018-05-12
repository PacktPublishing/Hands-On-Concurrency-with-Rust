#ifndef SIM_H
#define SIM_H

#include <stdint.h>

/* Instructions in core: */
typedef struct insn_st {
  uint16_t a, b;
  uint16_t in;
} insn_t;

/* Warrior data struct */
typedef struct warrior_st {
  insn_t*  code;                /* code of warrior */
  uint32_t len;                 /* length of warrior */
} warrior_t;

/* active warrior struct, only used inside of sim_proper() */
typedef struct w_st {
  insn_t**     tail;            /* next free location to queue a process */
  insn_t**     head;            /* next process to run from queue */
  uint32_t     nprocs;          /* number of live processes in this warrior */
  struct w_st* succ;            /* next warrior alive */
  struct w_st* pred;            /* previous warrior alive */
  uint32_t     id;              /* index (or identity) of warrior */
} w_t;

/* whole data needed by one simulator */
typedef struct mars_st {
  uint32_t nWarriors;

  uint32_t cycles;
  uint16_t coresize;
  uint32_t processes;

  uint16_t maxWarriorLength;

  w_t*     warTab;
  insn_t*  coreMem;
  insn_t** queueMem;
} mars_t;

void sim_free_bufs(mars_t* mars);
void sim_clear_core(mars_t* mars);

int sim_alloc_bufs(mars_t* mars);
int sim_load_warrior(mars_t* mars, uint32_t pos, const insn_t* const code, uint16_t len);
int sim_mw(mars_t* mars, const uint16_t * const war_pos_tab, uint32_t *death_tab );

#endif
