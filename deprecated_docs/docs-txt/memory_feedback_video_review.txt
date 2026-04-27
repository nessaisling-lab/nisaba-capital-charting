---
name: Video review procedure
description: Always extract frames from MP4 video clips using ffmpeg before reviewing; never try to Read binary MP4 directly
type: feedback
---

When the user provides an MP4 video clip for review, extract frames using ffmpeg (fps=1/3 or similar), then Read the resulting JPG frames. Never attempt to use the Read tool directly on MP4 files. The user has corrected this multiple times.

**Why:** The Read tool cannot handle binary MP4 files. ffmpeg frame extraction is the established workflow.

**How to apply:** Any time the user shares an MP4 path, immediately run:
```
ffmpeg -i "<path>" -vf "fps=1/3,scale=1280:-1" -q:v 3 "<output_dir>/review_frames_%03d.jpg" 2>&1
```
Then Read the resulting JPG files to visually review the dashboard.
