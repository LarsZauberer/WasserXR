#ifndef TS_CAMERA_H
#define TS_CAMERA_H

typedef struct {
  float fov;
  float near;
  float far;
} TS_Camera;

void *ts_create_TS_Camera();
void ts_destroy_TS_Camera(void *);

#endif
