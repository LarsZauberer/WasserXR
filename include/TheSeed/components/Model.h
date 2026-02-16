#ifndef TS_MODEL_H
#define TS_MODEL_H

#include "TheSeed/core/Shader.h"
typedef struct {
  TS_Shader *shader;
} TS_Model;

void *ts_create_TS_Model();
void ts_destroy_TS_Model(void *);

#endif
