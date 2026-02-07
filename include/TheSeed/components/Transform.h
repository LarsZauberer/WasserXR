#ifndef TS_Transform_H
#define TS_Transform_H

typedef struct {
  float x;
  float y;
  float z;
} TS_Transform_t;

void *ts_create_TS_Transform();
void ts_destroy_TS_Transform(void *);

#endif
