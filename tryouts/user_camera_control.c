#include <stdio.h>
#include <sys/ioctl.h>
#include <sys/mman.h>
#include <fcntl.h>
#include <linux/videodev2.h>
#include <linux/media.h>
#include <string.h>
#include <stdlib.h>
#include <unistd.h>
#include <sys/time.h>
void print_capabilities(unsigned int cap);
void print_pixelformat(unsigned int fmt);
void print_field(unsigned int fd);
void print_colorspace(unsigned int cb);

int main() {
    // open camera device
    int media_fd = open("/dev/video0", O_RDWR | O_NONBLOCK);
    printf("camera device fd: %d\n", media_fd);

    // query capabilities
    struct v4l2_capability info;

    if (ioctl(media_fd, VIDIOC_QUERYCAP, &info)) {
        perror("Get device info [FAILED]");
        return -1;
    } else {
        printf("Device info:\n");
        printf("\tdriver: %s\n", info.driver);
        printf("\tcard: %s\n", info.card);
        printf("\tbus_info: %s\n", info.bus_info);
        printf("\tversion: %u.%u.%u\n", (info.version >> 16) & 0xFF, (info.version >> 8) & 0xFF, info.version & 0xFF);
        printf("\tcapabilities:\n");
        print_capabilities(info.capabilities);
        printf("\tdevice_caps:\n");
        print_capabilities(info.device_caps);
    }

    // check video input
    int index;
    if (-1 == ioctl(media_fd, VIDIOC_G_INPUT, &index)) {
        perror("VIDIOC_G_INPUT [FAILED]");
        return -1;
    } else {
        struct v4l2_input input;
        memset(&input, 0, sizeof(input));
        input.index = index;

        if (-1 == ioctl(media_fd, VIDIOC_ENUMINPUT, &input)) {
            perror("VIDIOC_ENUMINPUT [FAILED]");
            return -1;
        }
        printf("Current input: %s\n", input.name);
    }

    // check image format
    struct v4l2_format format;
    format.type = V4L2_BUF_TYPE_VIDEO_CAPTURE;
    if (-1 == ioctl(media_fd, VIDIOC_G_FMT, &format)) {
        perror("VIDIOC_G_FMT [FAILED]");
        return -1;
    } else {
        printf("Image format:\n");
        printf("\twidth: %u\n", format.fmt.pix.width);
        printf("\theight: %u\n", format.fmt.pix.height);
        print_pixelformat(format.fmt.pix.pixelformat);
        print_field(format.fmt.pix.field);
        printf("\tbytesperline: %u\n", format.fmt.pix.bytesperline);
        printf("\tsizeimage: %u\n", format.fmt.pix.sizeimage);
        print_colorspace(format.fmt.pix.colorspace);
    }

    // check supported image format
    struct v4l2_fmtdesc fmtdesc;
    memset(&fmtdesc, 0, sizeof(fmtdesc));
    fmtdesc.type = V4L2_BUF_TYPE_VIDEO_CAPTURE;
    printf("Supported image format:\n");
    while (ioctl(media_fd, VIDIOC_ENUM_FMT, &fmtdesc) == 0) {
        printf("\t%s\n", fmtdesc.description);
        print_pixelformat(fmtdesc.pixelformat);
        fmtdesc.index++;
    }

    // switch to YUYV format
    format.fmt.pix.pixelformat = 0x56595559; // 'V', 'Y', 'U', 'Y'
    if (-1 == ioctl(media_fd, VIDIOC_S_FMT, &format)) {
        perror("VIDIOC_S_FMT [FAILED]");
        return -1;
    } else {
        if (-1 == ioctl(media_fd, VIDIOC_G_FMT, &format)) {
            perror("VIDIOC_G_FMT [FAILED]");
        }
    }

    /* start capture and get camera data */


    

    close(media_fd);
    return 0;
}