#! /bin/bash

ffmpeg \
-thread_queue_size 1024 \
-f alsa -guess_layout_max 0 \
-thread_queue_size 512 \
-f v4l2 -video_size 800x600 -framerate 25 -i /dev/video0 \
-c:v libx264 -pix_fmt yuv420p -preset veryfast -g 50 \
-b:v 2500k -maxrate 2500k -bufsize 7500k \
-acodec aac \
-b:a 32k \
-f flv "$1"


# -strict experimental is not needed. Users often add it for no reason.
#    AAC is recommended over MP3 by most streaming services.
#    -q:v is ignored by libx264, and anyway you already set a video bitrate control method with -b:v.
#    -q:a and -b:a are mutually exclusive options for audio bitrate control. For streaming prefer -b:a.
#    You can choose the audio sample rate (-sample_rate) at the ALSA level instead of converting it during encoding. See ffmpeg -h demuxer=alsa.
#    Set frame rate (-framerate) at the V4L2 level instead of converting it during encoding. See ffmpeg -h demuxer=v4l2. Your camera supports up to 25 fps at 800x600.
#    You can remove -threads 4 and let the encoder automatically choose the optimal number.
#    -bufsize can be bigger. 1-5x -maxrate.
#    -g can be 2x -framerate.
