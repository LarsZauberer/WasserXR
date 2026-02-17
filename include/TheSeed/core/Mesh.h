#ifndef TS_MESH_H
#define TS_MESH_H

#include <stddef.h>
typedef struct {
  size_t vertices_size;
  size_t faces_size;

  float *vertices;
  float *normals;
  unsigned int *indices;
} TS_Mesh_Data;

typedef struct {
  unsigned int numIndices;
  unsigned int vao;
  unsigned int vbo;
  unsigned int ebo;
} TS_Mesh;

unsigned int ts_read_mesh_data(TS_Mesh_Data *out, char *filename);
void ts_destroy_mesh_data(TS_Mesh_Data *mesh);

TS_Mesh *ts_create_mesh_from_data(TS_Mesh_Data *mesh_data);
void ts_destroy_mesh(TS_Mesh *mesh);

#endif
