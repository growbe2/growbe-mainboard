#include "i2c_comboard.h"
#include "i2c.h"
#include <stdio.h>

rs_cb_module_state_changed callback_state_changed;
rs_cb_module_value_validation callback_value_validation;
rs_cb_module_config_queue callback_config_queue;

int32_t register_callback(rs_cb_module_state_changed callback, rs_cb_module_value_validation c2, rs_cb_module_config_queue c3) {
    callback_state_changed = callback;
    callback_value_validation = c2;
    callback_config_queue = c3;
    return 1;
}

uint8_t buffer[512] = { 23, 23, 54};

Module_Config config;

void comboard_loop_body() {
    callback_state_changed(3, "AAAB0000003", true);

    callback_value_validation(3, buffer);

    callback_config_queue(&config);
    printf("Reiceve config for port %d\n", config.port);

    callback_state_changed(3, "AAAB0000003", false);

}