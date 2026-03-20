#ifndef TS_MESH_DATA_H
#define TS_MESH_DATA_H

#include <stddef.h>

typedef struct {
  unsigned int vertices_size;
  unsigned int faces_size;

  float *vertices;
  float *normals;
  float *uvs;
  unsigned int *indices;
} TS_Mesh_Data;

TS_Mesh_Data *ts_read_mesh_data(unsigned int *n, char *filename);
void ts_destroy_mesh_data(TS_Mesh_Data *mesh);

#endif
