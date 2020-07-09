/**
 * @file debug.h
 * @brief Everything related to the integrated debugger.
 * @details Currently only contains print_instruction method
 */
#pragma once

#include "tx8/core/instruction.h"

#include <tx8/core/cpu.h>

void tx_debug_print_instruction(tx_CPU* cpu, tx_Instruction* inst);
