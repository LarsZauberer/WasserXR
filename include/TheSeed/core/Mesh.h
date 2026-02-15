#ifndef TS_MESH_H
#define TS_MESH_H

#include <stddef.h>
typedef struct {
  size_t vertices_size;
  size_t indices_size;

  float *vertices;
  unsigned int *indices;
  float *texture_coordinates;
} TS_Mesh_Data;

typedef struct {
  TS_Mesh_Data *mesh;
  unsigned int vao;
  unsigned int vbo;
  unsigned int ebo;
} TS_Mesh;

TS_Mesh_Data *ts_read_model_file(char *filename);
void ts_destroy_mesh_data(TS_Mesh_Data *mesh);

TS_Mesh *ts_create_mesh_from_data(TS_Mesh_Data *mesh_data);
void ts_destroy_mesh(TS_Mesh *mesh);

#endif
