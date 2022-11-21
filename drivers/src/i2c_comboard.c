#include "i2c_comboard.h"
#include "i2c.h"
#include <stdio.h>

#define MEMORY_MODULE_ADD 		0x50U
// I2C addresses for the Communication board.
#define COM_BOARD_EEPROM		0x52U
#define COM_BOARD_BUS_SEL		0x77U
#define COM_BOARD_LEDS   		0x22U
#define COM_BOARD_LOOPS 		0x73U

// I2C addresses for the Power Bar 110v.
#define PWR_BOARD_EEPROM 		0x50U
#define PWR_BOARD_RELAY  		0x70U

// I2C addresses for the THL MODULE.
#define THL_BOARD_EEPROM 		0x40U
#define THL_BOARD_EMU_EEPROM_1  0x33U
#define THL_BOARD_EMU_EEPROM_2  0x34U

// I2C addresses for the SOIL MODULE.
#define SOIL_BOARD_EEPROM 		0x30U
#define SOIL_BOARD_EMU_EEPROM_1  0x31U
#define SOIL_BOARD_EMU_EEPROM_2  0x32U

static uint8_t txData[2];
static uint8_t rxData[512];
static uint8_t txDataWater[512];


static Module_Config config;

static int bus;
static I2CDevice dev;

enum color {
    YELLOW = 0x00,
    GREEN = 0x01
};
enum overwrite {
	STAYACTIVEOTHER = 0x00,
	CLOSEOTHER = 0x01
};
enum portstate {
	ONLINE = 0,
	OFFLINE = 1
};

typedef struct ModuleInfo {
	char id[17];
	int port;
	int connected;
} ModuleInfo;

static ModuleInfo module_ports[8] = {0};

rs_cb_module_state_changed callback_state_changed;
rs_cb_module_value_validation callback_value_validation;
rs_cb_module_config_queue callback_config_queue;


int I2cComLib_Write(char slaveAdd , uint8_t *data , int dataSize)
{
    dev.addr = slaveAdd;
    return i2c_write(&dev, 0x0, data, dataSize);
}

int I2cComLib_Read(char slaveAdd, uint8_t *data, int dataSize)
{
    dev.addr = slaveAdd;
    return i2c_ioctl_read(&dev, 0x00, data, dataSize);
}

void I2cComLib_CloseAllComPort(void)
{
    txData[0] = 0x00;
    txData[1] = 0x00;

    I2cComLib_Write (COM_BOARD_BUS_SEL, txData,1);
}

void I2cComLib_EnableComPort(char ComChannel)
{
    uint8_t txDataChannPort[1];

    switch (ComChannel) // MODIFIER POUR REPRÉSENTER LES PORT DE 0 @ 7 SUIVANT DE GAUCHE A DROITE DU BOARD
    {					// DANS LES FAIT LES PORT 0,1,2,3 & 4,5,6,7 SONT INVERSÉ
    	case 4 :
    		txDataChannPort[0] = 0x01;
    		break;
    	case 5 :
    		txDataChannPort[0] = 0x02;
    		break;
    	case 6 :
    		txDataChannPort[0] = 0x04;
    		break;
    	case 7 :
    		txDataChannPort[0] = 0x08;
    		break;
    	case 0 :
    		txDataChannPort[0] = 0x10;
    		break;
    	case 1 :
    		txDataChannPort[0] = 0x20;
    		break;
    	case 2 :
    		txDataChannPort[0] = 0x40;
    		break;
    	case 3 :
    		txDataChannPort[0] = 0x80;
    		break;
    	default :
    	break;
    }

    I2cComLib_Write (COM_BOARD_BUS_SEL, txDataChannPort,1);
}


// missing the parameters to store the ID of the module
int I2cComLib_ReadMemoryInfo(int deviseAddress, long dumpSize, char* id)
{
	uint8_t rxMemoryDataInfo[64] = {0};
	long eepromAddr = 0;

	uint8_t wSeq[2];

	wSeq[0] = (uint8_t)((eepromAddr >> 8) & 0XFF);
	wSeq[1] = (uint8_t)((eepromAddr & 0xFF));

	I2cComLib_Write(deviseAddress, wSeq , 2);

    int read = I2cComLib_Read(deviseAddress, rxMemoryDataInfo, dumpSize);

	char curInfo[20];

	if ( read > -1) {

		for (char i = 0; i < 16; ++i) {
			
		   	if (isprint(rxMemoryDataInfo[i]) != 0)
		    {
		        id[i] = rxMemoryDataInfo[i] ;
		    }
		}
	}
	return read > -1;
}



void I2cComLib_ClearAllYellowLed(void)	//PERMET DE CLEAR TOUT LES LED JAUNE SANS CHANGER LE STATE DES VERTES
{
	uint8_t txDataYellowLed[3];
	uint8_t currentLedState[2];

	txData[0] = 0x00;	//REGISTRE POUR LIRE LES VALEUR DE OUTPUT DES LED JAUNE ET VERT

	I2cComLib_Write(COM_BOARD_LEDS, txData, 1);

    I2cComLib_Read(COM_BOARD_LEDS, currentLedState, 2);

	currentLedState[0] = (currentLedState[0] | 0X55);
	currentLedState[1] = (currentLedState[1] | 0X55);

	txDataYellowLed[0] = 0x02;	//REGISTRE POUR ÉCRIRE SUR LES LED OUTPUT
	txDataYellowLed[1] = currentLedState[0];
	txDataYellowLed[2] = currentLedState[1];

	I2cComLib_Write(COM_BOARD_LEDS, txDataYellowLed, 3);
}

void I2cComLib_ClearAllGreenLed(void)	//PERMET DE CLEAR TOUT LES LED VERTE SANS CHANGER LE STATE DES JAUNES
{
	uint8_t txDataGreenLed[3];
	uint8_t currentLedState[2];

	txData[0] = 0x00;	//REGISTRE POUR LIRE LES VALEUR DE OUTPUT DES LED JAUNE ET VERT

	I2cComLib_Write(COM_BOARD_LEDS, txData, 1);

    I2cComLib_Read(COM_BOARD_LEDS, currentLedState, 2);

	currentLedState[0] = (currentLedState[0] | 0XAA);	//MASK DE TOUT LES LED JAUNE PORT 0
	currentLedState[1] = (currentLedState[1] | 0XAA);	//MASK DE TOUT LES LED JAUNE PORT 1

	txDataGreenLed[0] = 0x02;	//REGISTRE POUR ÉCRIRE SUR LES LED OUTPUT
	txDataGreenLed[1] = currentLedState[0];
	txDataGreenLed[2] = currentLedState[1];

	I2cComLib_Write (COM_BOARD_LEDS, txDataGreenLed,3);
}


void I2cComLib_EnableSoloLed(char comPort,enum portstate PortState,enum overwrite Overwrite, enum color Color) //Active une led en solo et ferme l'autre led du meme port
{
    uint8_t txDataLed[3];
    uint8_t txDataOtherLed[3];

    txDataLed[0] = 0x02;

    uint8_t currentLedState[2];

    txData[0] = 0x00;	//REGISTRE POUR LIRE LES VALEUR DE OUTPUT DES LED JAUNE ET VERT

    I2cComLib_Write(COM_BOARD_LEDS, txData, 1);

    I2cComLib_Read(COM_BOARD_LEDS, currentLedState, 2);

    if (Color == YELLOW)
    {
    	switch (comPort)
    	{
    	case 4:
    		txDataLed[1] = 0XFE;	// MODIFIER POUR REPRÉSENTER LES PORT DE 0 @ 7 SUIVANT DE GAUCHE A DROITE DU BOARD
    		txDataLed[2] = 0XFF;	// DANS LES FAIT LES PORT 0,1,2,3 & 4,5,6,7 SONT INVERSÉ

    		txDataOtherLed[1] = 0XFD;
    		txDataOtherLed[2] = 0XFF;
    		break;
    	case 5:
    		txDataLed[1] = 0XFB;
    		txDataLed[2] = 0XFF;

    		txDataOtherLed[1] = 0XF7;
    		txDataOtherLed[2] = 0XFF;
    		break;
    	case 6:
    		txDataLed[1] = 0XEF;
    		txDataLed[2] = 0XFF;

    		txDataOtherLed[1] = 0XDF;
    		txDataOtherLed[2] = 0XFF;
    		break;
    	case 7:
    		txDataLed[1] = 0XBF;
    		txDataLed[2] = 0XFF;

    		txDataOtherLed[1] = 0X7F;
    		txDataOtherLed[2] = 0XFF;
    		break;
    	case 0:
    		txDataLed[1] = 0XFF;
    		txDataLed[2] = 0XFE;

    		txDataOtherLed[1] = 0XFF;
    		txDataOtherLed[2] = 0XFD;
    		break;
    	case 1:
    		txDataLed[1] = 0XFF;
    		txDataLed[2] = 0XFB;

    		txDataOtherLed[1] = 0XFF;
    		txDataOtherLed[2] = 0XF7;
    		break;
    	case 2:
    		txDataLed[1] = 0XFF;
    		txDataLed[2] = 0XEF;

    		txDataOtherLed[1] = 0XFF;
    		txDataOtherLed[2] = 0XDF;
    		break;
    	case 3:
    		txDataLed[1] = 0XFF;
    		txDataLed[2] = 0XBF;

    		txDataOtherLed[1] = 0XFF;
    		txDataOtherLed[2] = 0X7F;
    		break;
    	default:
    		break;
    	}
    }

    else if (Color == GREEN)
    {
    	switch (comPort)
    	{
        	case 4:
        		txDataLed[1] = 0XFD;
        		txDataLed[2] = 0XFF;

        		txDataOtherLed[1] = 0XFE;
        		txDataOtherLed[2] = 0XFF;
    		break;
        	case 5:
        		txDataLed[1] = 0XF7;
        		txDataLed[2] = 0XFF;

        		txDataOtherLed[1] = 0XFB;
        		txDataOtherLed[2] = 0XFF;
    		break;
        	case 6:
        		txDataLed[1] = 0XDF;
        		txDataLed[2] = 0XFF;

        		txDataOtherLed[1] = 0XEF;
        		txDataOtherLed[2] = 0XFF;
    		break;
        	case 7:
        		txDataLed[1] = 0X7F;
        		txDataLed[2] = 0XFF;

        		txDataOtherLed[1] = 0XBF;
        		txDataOtherLed[2] = 0XFF;
    		break;
        	case 0:
        		txDataLed[1] = 0XFF;
        		txDataLed[2] = 0XFD;

        		txDataOtherLed[1] = 0XFF;
        		txDataOtherLed[2] = 0XFE;
    		break;
	        case 1:
	        	txDataLed[1] = 0XFF;
	        	txDataLed[2] = 0XF7;

        		txDataOtherLed[1] = 0XFF;
        		txDataOtherLed[2] = 0XFB;
    		break;
    	case 2:
        		txDataLed[1] = 0XFF;
        		txDataLed[2] = 0XDF;

        		txDataOtherLed[1] = 0XFF;
        		txDataOtherLed[2] = 0XEF;
    		break;
    	case 3:
        		txDataLed[1] = 0XFF;
        		txDataLed[2] = 0X7F;

        		txDataOtherLed[1] = 0XFF;
        		txDataOtherLed[2] = 0XBF;
    		break;
    	default:
    		break;
    	}
    }

    if (PortState == ONLINE)
    {

    }

    if (Overwrite == STAYACTIVEOTHER)
    {

        txDataLed[1] = ~txDataLed[1];
        txDataLed[2] = ~txDataLed[2];
        currentLedState[0] = ~currentLedState[0];
        currentLedState[1] = ~currentLedState[1];


    	currentLedState[0] = txDataOtherLed[1] & currentLedState[0];
    	currentLedState[1] = txDataOtherLed[2] & currentLedState[1];

        //txDataLed[0] = 0x02;	//REGISTRE POUR ÉCRIRE SUR LES LED OUTPUT
        txDataLed[1] =  txDataLed[1] | currentLedState[0];
        txDataLed[2] =  txDataLed[2] | currentLedState[1];

        txDataLed[1] = ~txDataLed[1];
        txDataLed[2] = ~txDataLed[2];

    }

    I2cComLib_Write (COM_BOARD_LEDS, txDataLed,3);
}

int i = 0;

void I2cComLib_SingleReadPortModuleInfo(int32_t device_index, char comPort) //PREMIERE FONCTION QUI VA CHERCHER LES INFOS DE LA MEMOIRE DES MODULEs
{

	bool result = false;


	ModuleInfo* info = &module_ports[comPort];

	info->id[16] = '\0';


	I2cComLib_EnableComPort(comPort);
	result = I2cComLib_ReadMemoryInfo(MEMORY_MODULE_ADD,64, info->id); // PEUT ALLER JUSQUA 128 de dump size
	I2cComLib_CloseAllComPort();

	if (result != info->connected || (info->connected == -1 && result == true)) {
		info->connected = result;
		if(result == true)
		{
			I2cComLib_EnableSoloLed(comPort,ONLINE,STAYACTIVEOTHER, GREEN);
			callback_state_changed(device_index, comPort, info->id, true);
		}
		else
		{
			I2cComLib_EnableSoloLed(comPort,OFFLINE,STAYACTIVEOTHER, YELLOW);
			callback_state_changed(device_index, comPort, info->id, false);
		}
	}
}



// PUBLIC FONCTION FOR RUST



int32_t register_callback_comboard(rs_cb_module_state_changed callback, rs_cb_module_value_validation c2, rs_cb_module_config_queue c3) {
    callback_state_changed = callback;
    callback_value_validation = c2;
    callback_config_queue = c3;
    return 1;
}



int init(const char* device) {
    if ((bus = i2c_open(device)) == -1) {
        return bus;
    }

    dev.bus = bus;
    dev.addr = 0x77U;
    dev.tenbit = 0;
    dev.delay = 10;
    dev.flags = 0;
    dev.page_bytes = 8;
    dev.iaddr_bytes = 0;

	module_ports[0].connected = -1;
	module_ports[1].connected = -1;
	module_ports[2].connected = -1;
	module_ports[3].connected = -1;
	module_ports[4].connected = -1;
	module_ports[5].connected = -1;
	module_ports[6].connected = -1;
	module_ports[7].connected = -1;


	uint8_t da[3];

	da[0] = 0x06;
	da[1] = 0x00;
	da[2] = 0x00;

	I2cComLib_Write (COM_BOARD_LEDS, da,3);

	da[0] = 0x02;
	da[1] = 0xFF;
	da[2] = 0xFF;
	I2cComLib_Write (COM_BOARD_LEDS, da,3);


	I2cComLib_ClearAllGreenLed();
	I2cComLib_ClearAllGreenLed();


    return bus;
}


void comboard_loop_body(int32_t device_index, int32_t starting_port, int32_t ending_port) {
	static uint8_t da[512];

	int data_read = -1;

    for (char comport = starting_port; comport < ending_port; ++comport)
    {
    	I2cComLib_SingleReadPortModuleInfo(device_index, comport);
    }

	Module_Config config;

	callback_config_queue(device_index, &config);

	for (int comport = starting_port; comport < ending_port; ++comport) {
		data_read = -1;
		if (module_ports[comport].connected == true) {
			I2cComLib_EnableComPort(comport);
			switch (module_ports[comport].id[2])
			{
				case 'A':
				{
					data_read = I2cComLib_Read (THL_BOARD_EMU_EEPROM_1, da, 512);
					break;
				}

				case 'B':
				{
					data_read = I2cComLib_Read(0x39, da, 512);
					
					if (config.port == comport) {
						for (int i = 0; i < 8; i++) {
							if (config.buffer[i] != 0xFF) {
								da[i] = config.buffer[i];
							}
						}
						I2cComLib_Write(0x40, da, 512);
					}
					break;
				}
				case 'P':
				{
					data_read = I2cComLib_Read(0x42, da, 512);
					
					if (config.port == comport) {
						for (int i = 0; i < 8; i++) {
							if (config.buffer[i] != 0xFF) {
								da[i] = config.buffer[i];
							}
						}
						I2cComLib_Write(0x43, da, 512);
					}
					
					break;
				}
				case 'S':
				{
					data_read = I2cComLib_Read(SOIL_BOARD_EEPROM, da, 512);
				}
			}
			if (data_read > -1) {
				callback_value_validation(device_index, comport, da);
			}

			I2cComLib_CloseAllComPort();
		}
	}
}
