#include "tx8/asm/types.h"

#include <stdlib.h>
#include <tx8/core/instruction.h>
#include <tx8/core/types.h>
#include <tx8/core/log.h>

void tx_asm_print_parameter(tx_Parameter* p) {
    switch (p->mode) {
        case tx_param_constant8: tx_log("0x%xu8", (tx_uint8)p->value.u); break;
        case tx_param_constant16: tx_log("0x%xu16", (tx_uint16)p->value.u); break;
        case tx_param_constant32: tx_log("0x%xu32", p->value.u); break;
        case tx_param_absolute_address: tx_log("#%x", p->value.u); break;
        case tx_param_relative_address:
            if (p->value.i < 0) tx_log("$-%x", -p->value.i);
            else
                tx_log("$%x", p->value.i);
            break;
        case tx_param_register_address: tx_log("@%s", tx_reg_names[p->value.u]); break;
        case tx_param_register: tx_log("%s", tx_reg_names[p->value.u]); break;
        default: tx_log("{0x%x}", p->value.u); break;
    }
}

static inline tx_uint32 tx_asm_param_size(tx_uint32 mode) {
    if (mode >= 0 && mode <= tx_param_register_address) return tx_param_sizes[mode];
    if (mode == tx_param_label) return tx_param_sizes[tx_param_constant32];
    return 0xffffffff;
}

tx_uint8 tx_asm_parameter_generate_binary(tx_Parameter* p, tx_uint8* buf) {
    switch (p->mode) {
        case tx_param_constant8:
        case tx_param_register:
        case tx_param_register_address: buf[0] = (tx_uint8)p->value.u; break;
        case tx_param_constant16: ((tx_uint16*)(buf))[0] = (tx_uint16)p->value.u; break;
        case tx_param_constant32:
        case tx_param_absolute_address:
        case tx_param_relative_address: ((tx_uint32*)(buf))[0] = (tx_uint32)p->value.u; break;
        default: break;
    }

    return tx_asm_param_size(p->mode);
}

tx_uint32 tx_asm_instruction_length(tx_Instruction* inst) {
    return 1 + tx_param_mode_bytes[tx_param_count[inst->opcode]] + tx_asm_param_size(inst->params.p1.mode)
           + tx_asm_param_size(inst->params.p2.mode);
}

void tx_asm_print_instruction(tx_Instruction* inst) {
    tx_log("%s", tx_op_names[inst->opcode]);
    if (tx_param_count[inst->opcode] > 0) {
        tx_log(" ");
        tx_asm_print_parameter(&(inst->params.p1));
    }
    if (tx_param_count[inst->opcode] > 1) {
        tx_log(" ");
        tx_asm_print_parameter(&(inst->params.p2));
    }
    tx_log("\n");
}

void tx_asm_instruction_generate_binary(tx_Instruction* inst, tx_uint8* buf) {
    buf[0] = inst->opcode;
    if (inst->params.p1.mode != 0) buf[1] = ((inst->params.p1.mode) << 4) | inst->params.p2.mode;

    tx_uint8* _buf = buf + 1 + tx_param_mode_bytes[tx_param_count[inst->opcode]];
    _buf += tx_asm_parameter_generate_binary(&(inst->params.p1), _buf);
    tx_asm_parameter_generate_binary(&(inst->params.p2), _buf);
}
