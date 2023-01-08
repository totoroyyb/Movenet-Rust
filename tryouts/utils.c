#include <linux/videodev2.h>
#include <stdio.h>
#include <string.h>
#include <stdlib.h>
void print_capabilities(unsigned int cap) {
    // https://dri.freedesktop.org/docs/drm/media/uapi/v4l/vidioc-querycap.html#device-capabilities
    if (cap & V4L2_CAP_VIDEO_CAPTURE) {
        printf("\t\tV4L2_CAP_VIDEO_CAPTURE\n");
    }
    if (cap & V4L2_CAP_VIDEO_CAPTURE_MPLANE) {
        printf("\t\tV4L2_CAP_VIDEO_CAPTURE_MPLANE\n");
    }
    if (cap & V4L2_CAP_VIDEO_OUTPUT) {
        printf("\t\tV4L2_CAP_VIDEO_OUTPUT\n");
    }
    if (cap & V4L2_CAP_VIDEO_OUTPUT_MPLANE) {
        printf("\t\tV4L2_CAP_VIDEO_OUTPUT_MPLANE\n");
    }
    if (cap & V4L2_CAP_VIDEO_M2M) {
        printf("\t\tV4L2_CAP_VIDEO_M2M\n");
    }
    if (cap & V4L2_CAP_VIDEO_M2M_MPLANE) {
        printf("\t\tV4L2_CAP_VIDEO_M2M_MPLANE\n");
    }
    if (cap & V4L2_CAP_VIDEO_OVERLAY) {
        printf("\t\tV4L2_CAP_VIDEO_OVERLAY\n");
    }
    if (cap & V4L2_CAP_VBI_CAPTURE) {
        printf("\t\tV4L2_CAP_VBI_CAPTURE\n");
    }
    if (cap & V4L2_CAP_VBI_OUTPUT) {
        printf("\t\tV4L2_CAP_VBI_OUTPUT\n");
    }
    if (cap & V4L2_CAP_SLICED_VBI_CAPTURE) {
        printf("\t\tV4L2_CAP_SLICED_VBI_CAPTURE\n");
    }
    if (cap & V4L2_CAP_SLICED_VBI_OUTPUT) {
        printf("\t\tV4L2_CAP_SLICED_VBI_OUTPUT\n");
    }
    if (cap & V4L2_CAP_RDS_CAPTURE) {
        printf("\t\tV4L2_CAP_RDS_CAPTURE\n");
    }
    if (cap & V4L2_CAP_VIDEO_OUTPUT_OVERLAY) {
        printf("\t\tV4L2_CAP_VIDEO_OUTPUT_OVERLAY\n");
    }
    if (cap & V4L2_CAP_HW_FREQ_SEEK) {
        printf("\t\tV4L2_CAP_HW_FREQ_SEEK\n");
    }
    if (cap & V4L2_CAP_RDS_OUTPUT) {
        printf("\t\tV4L2_CAP_RDS_OUTPUT\n");
    }
    if (cap & V4L2_CAP_TUNER) {
        printf("\t\tV4L2_CAP_TUNER\n");
    }
    if (cap & V4L2_CAP_AUDIO) {
        printf("\t\tV4L2_CAP_AUDIO\n");
    }
    if (cap & V4L2_CAP_RADIO) {
        printf("\t\tV4L2_CAP_RADIO\n");
    }
    if (cap & V4L2_CAP_MODULATOR) {
        printf("\t\tV4L2_CAP_MODULATOR\n");
    }
    if (cap & V4L2_CAP_SDR_CAPTURE) {
        printf("\t\tV4L2_CAP_SDR_CAPTURE\n");
    }
    if (cap & V4L2_CAP_EXT_PIX_FORMAT) {
        printf("\t\tV4L2_CAP_EXT_PIX_FORMAT\n");
    }
    if (cap & V4L2_CAP_SDR_OUTPUT) {
        printf("\t\tV4L2_CAP_SDR_OUTPUT\n");
    }
    if (cap & V4L2_CAP_META_CAPTURE) {
        printf("\t\tV4L2_CAP_META_CAPTURE\n");
    }
    if (cap & V4L2_CAP_READWRITE) {
        printf("\t\tV4L2_CAP_READWRITE\n");
    }
    if (cap & V4L2_CAP_ASYNCIO) {
        printf("\t\tV4L2_CAP_ASYNCIO\n");
    }
    if (cap & V4L2_CAP_STREAMING) {
        printf("\t\tV4L2_CAP_STREAMING\n");
    }
    if (cap & V4L2_CAP_META_OUTPUT) {
        printf("\t\tV4L2_CAP_META_OUTPUT\n");
    }
    if (cap & V4L2_CAP_TOUCH) {
        printf("\t\tV4L2_CAP_TOUCH\n");
    }
    if (cap & V4L2_CAP_DEVICE_CAPS) {
        printf("\t\tV4L2_CAP_DEVICE_CAPS\n");
    }
}

void print_pixelformat(unsigned int fmt) {
    // https://dri.freedesktop.org/docs/drm/media/uapi/v4l/pixfmt-reserved.html#reserved-formats
    printf("\tpixelformat: \n");
    printf("\t\t0x%x: %c%c%c%c\n", fmt, (char)(fmt & 0xFF), (char)(fmt>>8 & 0xFF), (char)(fmt>>16 & 0xFF), (char)((fmt>>24) & 0xFF));
}

void print_field(unsigned int fd) {
    // https://dri.freedesktop.org/docs/drm/media/uapi/v4l/field-order.html#c.v4l2_field
    printf("\tfield: \n");
    switch (fd) {
        case V4L2_FIELD_ANY:
            printf("\t\tV4L2_FIELD_ANY\n");
            break;
        case V4L2_FIELD_NONE:
            printf("\t\tV4L2_FIELD_NONE\n");
            break;
        case V4L2_FIELD_TOP:
            printf("\t\tV4L2_FIELD_TOP\n");
            break;
        case V4L2_FIELD_BOTTOM:
            printf("\t\tV4L2_FIELD_BOTTOM\n");
            break;
        case V4L2_FIELD_INTERLACED:
            printf("\t\tV4L2_FIELD_INTERLACED\n");
            break;
        case V4L2_FIELD_SEQ_TB:
            printf("\t\tV4L2_FIELD_SEQ_TB\n");
            break;
        case V4L2_FIELD_SEQ_BT:
            printf("\t\tV4L2_FIELD_SEQ_BT\n");
            break;
        case V4L2_FIELD_ALTERNATE:
            printf("\t\tV4L2_FIELD_ALTERNATE\n");
            break;
        case V4L2_FIELD_INTERLACED_TB:
            printf("\t\tV4L2_FIELD_INTERLACED_TB\n");
            break;
        case V4L2_FIELD_INTERLACED_BT:
            printf("\t\tV4L2_FIELD_INTERLACED_BT\n");
            break;
    }
}

void print_colorspace(unsigned int cb) {
    // https://dri.freedesktop.org/docs/drm/media/uapi/v4l/colorspaces-defs.html#c.v4l2_colorspace
    printf("\tcolor space: \n");
    switch (cb) {
        case V4L2_COLORSPACE_DEFAULT:
            printf("\t\tV4L2_COLORSPACE_DEFAULT\n");
            break;
        case V4L2_COLORSPACE_SMPTE170M:
            printf("\t\tV4L2_COLORSPACE_SMPTE170M\n");
            break;
        case V4L2_COLORSPACE_REC709:
            printf("\t\tV4L2_COLORSPACE_REC709\n");
            break;
        case V4L2_COLORSPACE_SRGB:
            printf("\t\tV4L2_COLORSPACE_SRGB\n");
            break;
        case V4L2_COLORSPACE_OPRGB:
            printf("\t\tV4L2_COLORSPACE_OPRGB\n");
            break;
        case V4L2_COLORSPACE_BT2020:
            printf("\t\tV4L2_COLORSPACE_BT2020\n");
            break;
        case V4L2_COLORSPACE_DCI_P3:
            printf("\t\tV4L2_COLORSPACE_DCI_P3\n");
            break;
        case V4L2_COLORSPACE_SMPTE240M:
            printf("\t\tV4L2_COLORSPACE_SMPTE240M\n");
            break;
        case V4L2_COLORSPACE_470_SYSTEM_M:
            printf("\t\tV4L2_COLORSPACE_470_SYSTEM_M\n");
            break;
        case V4L2_COLORSPACE_470_SYSTEM_BG:
            printf("\t\tV4L2_COLORSPACE_470_SYSTEM_BG\n");
            break;
        case V4L2_COLORSPACE_JPEG:
            printf("\t\tV4L2_COLORSPACE_JPEG\n");
            break;
        case V4L2_COLORSPACE_RAW:
            printf("\t\tV4L2_COLORSPACE_RAW\n");
            break;
    }
}

void save_bmp(char *file_name, unsigned char* buffer, int w, int h) {
    FILE *f;
    unsigned char *img = NULL;
    int filesize = 54 + 3 * w * h;

    img = (unsigned char *) malloc(3 * w * h);
    memset(img, 0, 3 * w * h);

    for (int i = 0; i < w; i++) {
        for (int j = 0; j < h; j++) {
            int x = i;
            // int y = (h - 1) - j;
            int y = j;
            img[(x + y * w) * 3 + 2] = buffer[(j * w + i) * 3 + 0];
            img[(x + y * w) * 3 + 1] = buffer[(j * w + i) * 3 + 1];
            img[(x + y * w) * 3 + 0] = buffer[(j * w + i) * 3 + 2];
        }
    }

    unsigned char bmpfileheader[14] = { 'B', 'M', 0, 0, 0, 0, 0, 0, 0, 0, 54, 0, 0, 0 };
    unsigned char bmpinfoheader[40] = { 40, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 24, 0 };
    unsigned char bmppad[3] = { 0, 0, 0 };

    bmpfileheader[2] = (unsigned char)(filesize);
    bmpfileheader[3] = (unsigned char)(filesize >> 8);
    bmpfileheader[4] = (unsigned char)(filesize >> 16);
    bmpfileheader[5] = (unsigned char)(filesize >> 24);

    bmpinfoheader[4] = (unsigned char)(w);
    bmpinfoheader[5] = (unsigned char)(w >> 8);
    bmpinfoheader[6] = (unsigned char)(w >> 16);
    bmpinfoheader[7] = (unsigned char)(w >> 24);
    bmpinfoheader[8] = (unsigned char)(h);
    bmpinfoheader[9] = (unsigned char)(h >> 8);
    bmpinfoheader[10] = (unsigned char)(h >> 16);
    bmpinfoheader[11] = (unsigned char)(h >> 24);

    f = fopen(file_name, "wb");
    fwrite(bmpfileheader, 1, 14, f);
    fwrite(bmpinfoheader, 1, 40, f);
    for (int i = 0; i < h; i++) {
        fwrite(img + (w * (h - i - 1) * 3), 3, w, f);
        fwrite(bmppad, 1, (4 - (w * 3) % 4) % 4, f);
    }

    free(img);
    fclose(f);
}