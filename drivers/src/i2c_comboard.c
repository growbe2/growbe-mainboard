#include "i2c_comboard.h"
#include "i2c.h"
#include <stdio.h>

rs_cb_module_state_changed callback_state_changed;

int32_t register_callback(rs_cb_module_state_changed callback) {
    callback_state_changed = callback;

    callback_state_changed(50);
    return 1;
}