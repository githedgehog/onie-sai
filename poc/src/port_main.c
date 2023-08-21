#include <stdio.h>
#include <stdlib.h>
#include <stdint.h>
#include <unistd.h>
#include <errno.h>
#include <string.h>

#define PHY_MAILBOX_MMD_OFFSET (256 + 250)
#define PHY_MAILBOX_DATA_OFFSET (256 + 253)
#define PHY_MAILBOX_CTRL_OFFSET (256 + 255)
#define PHY_MAILBOX_STATUS_OFFSET PHY_MAILBOX_CTRL_OFFSET
#define PHY_MAILBOX_RD_SIZE 3
#define PHY_MAILBOX_WR_SIZE 5
#define PHY_MAILBOX_STATUS_DONE 0x4
#define PHY_MAILBOX_STATUS_ERROR 0x8
#define PHY_MAILBOX_RD_BYTE 0x01
#define PHY_MAILBOX_WR_BYTE 0x02

#define PHY_MAILBOX_DIS_DELAY = 1   # 1000ms
#define PHY_MAILBOX_EN_DELAY = 0.01 #   10ms

/*
[bytes([0x07, 0x00, 0x20, 0x11, 0x83]), \
                            bytes([0x07, 0xFF, 0xE9, 0x02, 0x00]), \
                            bytes([0x07, 0xFF, 0xE4, 0x91, 0x01]), \
                            bytes([0x07, 0x00, 0x00, 0x32, 0x00])]
                            */
static int write_command(uint8_t *buf);
static int read_command(uint8_t *buf);

int main(int argc, char *argv[]) {
    // read admin state
    uint8_t rd0[] = {0x01, 0x00, 0x09};
    read_command(rd0);

    // read link state
    uint8_t rd1[] = {0x1E, 0x40, 0x0D};
    read_command(rd1);

    // disable admin state
    uint8_t buf0[] = {0x01, 0x00, 0x09, 0x00, 0x01};
    write_command(buf0);

    // set speed to 10G
    uint8_t buf1[] = {0x07, 0x00, 0x20, 0x11, 0x83};
    uint8_t buf2[] = {0x07, 0xFF, 0xE9, 0x02, 0x00};
    uint8_t buf3[] = {0x07, 0xFF, 0xE4, 0x91, 0x01};
    uint8_t buf4[] = {0x07, 0x00, 0x00, 0x32, 0x00};
    write_command(buf1);
    write_command(buf2);
    write_command(buf3);
    write_command(buf4);

    // enable admin state
    uint8_t buf5[] = {0x01, 0x00, 0x09, 0x00, 0x00};
    write_command(buf5);
}

static int write_command(uint8_t *buf) {
    const char *eeprom_path = "/sys/class/i2c-adapter/i2c-2/2-0050/eeprom"; // Replace with the actual path
    FILE *eeprom = fopen(eeprom_path, "rb+");
    
    if (eeprom == NULL) {
        perror("Error opening file");
        return 1;
    }
    
    // seek to where to write the command buffer and write it
    if (fseek(eeprom, PHY_MAILBOX_MMD_OFFSET, SEEK_SET) != 0) {
        perror("Error seeking in file to mmd offset");
        fclose(eeprom);
        return 1;
    }

    size_t buf_bytes_written = fwrite(buf, sizeof(uint8_t), PHY_MAILBOX_WR_SIZE, eeprom);
    if (buf_bytes_written != PHY_MAILBOX_WR_SIZE) {
        perror("Error writing command buffer to file");
        fclose(eeprom);
        return 1;
    }

    // Write the WRITE Control Byte, at the control offset
    if (fseek(eeprom, PHY_MAILBOX_CTRL_OFFSET, SEEK_SET) != 0) {
        perror("Error seeking in file to ctrl offset");
        fclose(eeprom);
        return 1;
    }

    uint8_t data_to_write = PHY_MAILBOX_WR_BYTE;
    size_t bytes_written = fwrite(&data_to_write, sizeof(uint8_t), 1, eeprom);
    if (bytes_written != 1) {
        perror("Error writing ctrl byte to file");
        fclose(eeprom);
        return 1;
    }

    // Read the status of MDIO operation
    usleep(100000);
    if (fseek(eeprom, PHY_MAILBOX_STATUS_OFFSET, SEEK_SET) != 0) {
        perror("Error seeking in file to status offset");
        fclose(eeprom);
        return 1;
    }

    uint8_t status;
    size_t status_bytes_read = fread(&status, sizeof(uint8_t), 1, eeprom);
    if (status_bytes_read != 1) {
        perror("Error reading status byte of file");
        fclose(eeprom);
        return 1;
    }

    if ((int)status & PHY_MAILBOX_STATUS_DONE) {
        printf("successfully written command (status 0x%x)\n", status);
    } else {
        printf("error writing command (status 0x%x)\n", status);
    }
    fclose(eeprom);
    return 0;
}

static int read_command(uint8_t *buf) {
    const char *eeprom_path = "/sys/class/i2c-adapter/i2c-2/2-0050/eeprom"; // Replace with the actual path
    FILE *eeprom = fopen(eeprom_path, "rb+");
    
    if (eeprom == NULL) {
        perror("Error opening file");
        return 1;
    }
    
    // seek to where to write the command buffer and write it
    if (fseek(eeprom, PHY_MAILBOX_MMD_OFFSET, SEEK_SET) != 0) {
        perror("Error seeking in file to mmd offset");
        fclose(eeprom);
        return 1;
    }

    size_t buf_bytes_written = fwrite(buf, sizeof(uint8_t), PHY_MAILBOX_RD_SIZE, eeprom);
    if (buf_bytes_written != PHY_MAILBOX_RD_SIZE) {
        perror("Error writing command buffer to file");
        fclose(eeprom);
        return 1;
    }

    // Write the WRITE Control Byte, at the control offset
    if (fseek(eeprom, PHY_MAILBOX_CTRL_OFFSET, SEEK_SET) != 0) {
        perror("Error seeking in file to ctrl offset");
        fclose(eeprom);
        return 1;
    }

    uint8_t data_to_write = PHY_MAILBOX_RD_BYTE;
    size_t bytes_written = fwrite(&data_to_write, sizeof(uint8_t), 1, eeprom);
    if (bytes_written != 1) {
        perror("Error writing ctrl byte to file");
        fclose(eeprom);
        return 1;
    }

    // Read the status of MDIO operation
    usleep(100000);
    if (fseek(eeprom, PHY_MAILBOX_STATUS_OFFSET, SEEK_SET) != 0) {
        perror("Error seeking in file to status offset");
        fclose(eeprom);
        return 1;
    }

    uint8_t status;
    size_t status_bytes_read = fread(&status, sizeof(uint8_t), 1, eeprom);
    if (status_bytes_read != 1) {
        perror("Error reading status byte of file");
        fclose(eeprom);
        return 1;
    }

    if ((int)status & PHY_MAILBOX_STATUS_DONE) {
        printf("successfully written command (status 0x%x)\n", status);
    } else {
        printf("error writing command (status 0x%x)\n", status);
        // fclose(eeprom);
        // return 1;
    }

    if (fseek(eeprom, PHY_MAILBOX_DATA_OFFSET, SEEK_SET) != 0) {
        perror("Error seeking in file to ctrl offset");
        fclose(eeprom);
        return 1;
    }

    uint8_t data[2] = {0};
    size_t data_bytes_read = fread(data, sizeof(uint8_t), 2, eeprom);
    if (data_bytes_read != 2) {
        perror("Error reading data bytes of file");
        fclose(eeprom);
        return 1;
    }

    printf("data read: 0x%x 0x%x\n", data[0], data[1]);

    fclose(eeprom);
    return 0;
}


/*
PHY_MAILBOX_MMD_OFFSET = 256 + 250
PHY_MAILBOX_DATA_OFFSET = 256 + 253
PHY_MAILBOX_CTRL_OFFSET = 256 + 255
PHY_MAILBOX_STATUS_OFFSET = PHY_MAILBOX_CTRL_OFFSET
PHY_MAILBOX_RD_SIZE = 3
PHY_MAILBOX_WR_SIZE = 5
#PHY_MAILBOX_RW_STATUS_IDLE = 0x0
PHY_MAILBOX_STATUS_DONE = 0x4
PHY_MAILBOX_STATUS_ERROR = 0x8
PHY_MAILBOX_RD_BYTE = bytes([0x01])
PHY_MAILBOX_WR_BYTE = bytes([0x02])

PHY_MAILBOX_DIS_DELAY = 1   # 1000ms
PHY_MAILBOX_EN_DELAY = 0.01 #   10ms

PHY_RD_TX_STATE_CMD = 0
PHY_WR_TX_DISABLE_CMD = 1
PHY_WR_TX_ENABLE_CMD = 2
PHY_WR_SPEED_1G_CMD = 3
PHY_WR_SPEED_10G_CMD = 4
PHY_RD_LINK_STATE_CMD = 5
PHY_WR_SPEED_ALL_CMD = 6


PHY_SPEED_AUTONEG = 0

autoneg_task = None
autoneg_lock = threading.Lock()
autoneg_ports = {}
stopping_event = threading.Event()
appl_db = None
state_db = None

def _phy_mailbox_read(eeprom_path, buf):
    eeprom = None
    value = 0
    ret = (False, value)

    try:
        if len(buf) != PHY_MAILBOX_RD_SIZE:
            return ret

        eeprom = open(eeprom_path, mode="rb+", buffering=0)

        # Write 3 bytes MMD and MDIO address
        eeprom.seek(PHY_MAILBOX_MMD_OFFSET)
        eeprom.write(buf)

        # Write the READ Control Byte
        eeprom.seek(PHY_MAILBOX_CTRL_OFFSET)
        eeprom.write(PHY_MAILBOX_RD_BYTE)

        # Read the status of MDIO operation
        eeprom.seek(PHY_MAILBOX_STATUS_OFFSET)
        status = int(hex(eeprom.read(1)[0]), 16)
        if status & PHY_MAILBOX_STATUS_DONE:
            # Read 2 bytes of data
            eeprom.seek(PHY_MAILBOX_DATA_OFFSET)
            data = eeprom.read(2)
            value = int(hex(data[1]), 16)
            value |= int(hex(data[0]), 16) << 8
            ret = (True, value)
        elif status & PHY_MAILBOX_STATUS_ERROR:
            pass
        else:
            # Retry status read after 100ms
            time.sleep(0.1)
            eeprom.seek(PHY_MAILBOX_STATUS_OFFSET)
            status = int(hex(eeprom.read(1)[0]), 16)
            if status & PHY_MAILBOX_STATUS_DONE:
                # Read 2 bytes of data
                eeprom.seek(PHY_MAILBOX_DATA_OFFSET)
                data = eeprom.read(2)
                value = int(hex(data[1]), 16)
                value |= int(hex(data[1]), 16) << 8
                ret = (True, value)
    except:
        pass

    if eeprom != None:
        eeprom.close()

    return ret

def _phy_mailbox_write(eeprom_path, buf):
    eeprom = None
    ret = False

    try:
        if len(buf) != PHY_MAILBOX_WR_SIZE:
            return ret

        eeprom = open(eeprom_path, mode="rb+", buffering=0)

        # Write 3 bytes MMD and MDIO address and 2 bytes of Data
        eeprom.seek(PHY_MAILBOX_MMD_OFFSET)
        eeprom.write(buf)

        # Write the WRITE Control Byte
        eeprom.seek(PHY_MAILBOX_CTRL_OFFSET)
        eeprom.write(PHY_MAILBOX_WR_BYTE)

        # Read the status of MDIO operation
        eeprom.seek(PHY_MAILBOX_STATUS_OFFSET)
        status = int(hex(eeprom.read(1)[0]), 16)
        if status & PHY_MAILBOX_STATUS_DONE:
            ret = True
        elif status & PHY_MAILBOX_STATUS_ERROR:
            pass
        else:
            # Retry status read after 100ms
            time.sleep(0.1)
            eeprom.seek(PHY_MAILBOX_STATUS_OFFSET)
            status = int(hex(eeprom.read(1)[0]), 16)
            if status & PHY_MAILBOX_STATUS_DONE:
                ret = True
    except:
        pass

    if eeprom != None:
        eeprom.close()

    return ret

*/