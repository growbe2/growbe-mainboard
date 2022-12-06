#ifndef I2C_COMBOARD_H_INCLUDED
#define I2C_COMBOARD_H_INCLUDED

#include <stdint.h>
#include <stdbool.h>

typedef struct Module_Config {
    int32_t port;
    uint8_t buffer[512];
} Module_Config;

typedef void (*rs_cb_module_state_changed)(int32_t device_index, int32_t port, const char* id, bool state);
typedef void (*rs_cb_module_value_validation)(int32_t device_index, int32_t port, uint8_t buffer[512]);
typedef void (*rs_cb_module_config_queue)(int32_t device_index, Module_Config* config);

void comboard_loop_body(int32_t device_index, int32_t starting_port, int32_t ending_port);
int init(const char* device);
int32_t register_callback_comboard(rs_cb_module_state_changed, rs_cb_module_value_validation, rs_cb_module_config_queue);

#endif /* LIB1_H_INCLUDED */
