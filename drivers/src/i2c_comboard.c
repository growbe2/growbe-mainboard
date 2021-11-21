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
    return i2c_read(&dev, 0x0, data, dataSize);
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
int I2cComLib_ReadMemoryInfo(int deviseAddress, long dumpSize)
{
	//uint8_t rxMemoryDataInfo[dumpSize];
	uint8_t rxMemoryDataInfo[64] = {0};
	//char  rdata[16] = {0};
	//long dSize = dumpSize;
	long eepromAddr = 0;
	//status_t status;
	//struct _ModuleData ModuleInfo;

	uint8_t wSeq[2];

	wSeq[0] = (uint8_t)((eepromAddr >> 8) & 0XFF);
	wSeq[1] = (uint8_t)((eepromAddr & 0xFF));

	I2cComLib_Write(deviseAddress, wSeq , 2);

    /*masterXfer.slaveAddress   = deviseAddress;
    masterXfer.direction      = kI2C_Read;
    masterXfer.subaddress     = 0;
    masterXfer.subaddressSize = 0;
    masterXfer.data           = rxMemoryDataInfo;
    masterXfer.dataSize       = dumpSize;
    masterXfer.flags          = kI2C_TransferDefaultFlag;
    status = I2C_RTOS_Transfer(&master_rtos_handle, &masterXfer);
    */
    I2cComLib_Read(deviseAddress, rxMemoryDataInfo, dumpSize);

    //console_print("Module info (No.Series) (Info): ");

    for (char i = 0; i < 16; ++i) {

    	//if (isprint(rxMemoryDataInfo[i]) != 0)
    	//{
            //curInfo->id[i] = rxMemoryDataInfo[i] ;
    	//}
    	//if (isprint(rxMemoryDataInfo[i+16]) != 0)
    	//{
            //curInfo->name[i] = rxMemoryDataInfo[i+16] ;
    	//}
	}


    return true;

}


void I2cComLib_EnableSoloLed(char comPort,enum portstate PortState,enum overwrite Overwrite, enum color Color) //Active une led en solo et ferme l'autre led du meme port
{
    uint8_t txDataLed[3];
    uint8_t txDataOtherLed[3];

    txDataLed[0] = 0x02;

    uint8_t currentLedState[2];

    txData[0] = 0x00;	//REGISTRE POUR LIRE LES VALEUR DE OUTPUT DES LED JAUNE ET VERT

    I2cComLib_Write(COM_BOARD_LEDS, txData, 1);

    /*masterXfer.slaveAddress   = COM_BOARD_LEDS;
    masterXfer.direction      = kI2C_Read;
    masterXfer.subaddress     = 0;
    masterXfer.subaddressSize = 0;
    masterXfer.data           = currentLedState;
    masterXfer.dataSize       = 2;
    masterXfer.flags          = kI2C_TransferDefaultFlag;
    status = I2C_RTOS_Transfer(&master_rtos_handle, &masterXfer);*/
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


void I2cComLib_SingleReadPortModuleInfo(char comPort) //PREMIERE FONCTION QUI VA CHERCHER LES INFOS DE LA MEMOIRE DES MODULEs
{

	bool result = false;

	I2cComLib_EnableComPort(comPort);
	result = I2cComLib_ReadMemoryInfo(MEMORY_MODULE_ADD,64); // PEUT ALLER JUSQUA 128 de dump size
	I2cComLib_CloseAllComPort();

	if(result == true)
	{
		I2cComLib_EnableSoloLed(comPort,ONLINE,STAYACTIVEOTHER, GREEN);
		//onModuleStateChange(comPort, true);
	}
	else
	{
		I2cComLib_EnableSoloLed(comPort,OFFLINE,STAYACTIVEOTHER, YELLOW);
		//onModuleStateChange(comPort, false);
	}

}



// PUBLIC FONCTION FOR RUST



int32_t register_callback(rs_cb_module_state_changed callback, rs_cb_module_value_validation c2, rs_cb_module_config_queue c3) {
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


    return bus;
}


void comboard_loop_body() {
    // valid is there is an action to accomplish (at the end)
    // callback_config_queue(&config);

    // Valide the state of all the port

    for (char comport = 0; comport < 8; ++comport)
    {
    	I2cComLib_SingleReadPortModuleInfo(comport);

    }
}