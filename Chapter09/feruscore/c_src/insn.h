#ifndef INSN_H
#define INSN_H
/* insn.h: Instruction encoding definition
 * $Id: insn.h,v 1.1 2003/08/02 07:40:36 martinus Exp $
 */

/* This file is part of `exhaust', a memory array redcode simulator.
 * Copyright (C) 2002 M Joonas Pihlaja
 * Public Domain.
 */

#include "insn_help.h"

/*
 * Instruction encoding:
 *
 * Instructions are held in a insn_t typed struct with three fields:
 *   in:    instruction flags, opcode, modifier, a-mode, b-mode
 *   a:     a-value
 *   b:     b-value
 *
 * The layout of the `in' field is as follows:
 *
 * bit         15 14 13 12 11 10  9  8  7  6  5  4  3  2  1  0
 * field   | flags | |- op-code  -| |-.mod-| |b-mode| |a-mode|
 *
 * Currently there is only one flag, the fl_START flag, which
 * the assembler uses to figure out the starting instruction
 * of a warrior (i.e. the one given by the START label).
 */

/*
 * Encodings for various fields of the `in' field.
 *
 */
enum ex_op {
    EX_DAT,             /* must be 0 */
    EX_SPL,
    EX_MOV,
    EX_DJN,
    EX_ADD,
    EX_JMZ,
    EX_SUB,
    EX_SEQ,
    EX_SNE,
    EX_SLT,
    EX_JMN,
    EX_JMP,
    EX_NOP,
    EX_MUL,
    EX_MODM,
    EX_DIV,             /* 16 */
};

enum ex_modifier {
    EX_mF, EX_mA, EX_mB, EX_mAB, EX_mBA, EX_mX, EX_mI   /* 7 */
};

enum ex_addr_mode {             /* must start from 0,
                                   the ordering is important */
    EX_DIRECT,  /* $ */
    EX_IMMEDIATE, /* # */
    EX_BINDIRECT,   /* @ */
    EX_BPREDEC, /* < */
    EX_BPOSTINC,    /* > */
    EX_AINDIRECT,   /* * */
    EX_APREDEC, /* { */
    EX_APOSTINC /* } */   /* 8 */
};

#define EX_INDIRECT EX_BINDIRECT
#define EX_PREDEC   EX_BPREDEC
#define EX_POSTINC  EX_BPOSTINC

#define EX_INDIR_A(mode) ((mode) >= EX_AINDIRECT)

/* mode */
#define EX_RAW_MODE(mode) ((mode) + (EX_INDIRECT-EX_AINDIRECT))


#endif /* INSN_H */
