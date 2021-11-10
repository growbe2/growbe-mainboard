#ifndef I2C_COMBOARD_H_INCLUDED
#define I2C_COMBOARD_H_INCLUDED

#include <stdint.h>

typedef void (*rs_cb_module_state_changed)(int32_t);

int32_t register_callback(rs_cb_module_state_changed);

#endif /* LIB1_H_INCLUDED */
