#ifndef TS_MESH_H
#define TS_MESH_H

#include "TheSeed/core/Mesh_Data.h"
#include <stddef.h>

typedef struct {
  int numIndices;
  unsigned int vao;
  unsigned int vbo;
  unsigned int ebo;
} TS_Mesh;

TS_Mesh *ts_create_mesh_from_data(TS_Mesh_Data *mesh_data);
void ts_destroy_mesh(TS_Mesh *mesh);

#endif
