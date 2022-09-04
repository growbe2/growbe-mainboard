

#include "i2c_comboard.h"


void state_changed(int32_t port, const char* id, bool state) {

}
void value_validation(int32_t port, uint8_t buffer[512]) {

}
void module_config_queue(Module_Config* config) {

}


int main() {

	register_callback_comboard(state_changed, value_validation, module_config_queue);
	init("/dev/i2c-1");

	while (1 != 0) {
		comboard_loop_body(0, 8);
	}

	return 0;
}