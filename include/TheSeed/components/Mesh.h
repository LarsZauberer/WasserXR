#ifndef TS_MESH_H
#define TS_MESH_H

#include "TheSeed/core/Shader.h"
typedef struct {
  float *vertices;
  unsigned int *indices;
  unsigned int vao;
  unsigned int vbo;
  unsigned int ebo;
  TS_Shader *shader;
} TS_Mesh;

void *ts_create_TS_Mesh();
void ts_destroy_TS_Mesh(void *);

#endif
