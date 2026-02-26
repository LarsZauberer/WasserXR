#ifndef TS_MODEL_H
#define TS_MODEL_H

#include "TheSeed/core/Mesh.h"
#include "TheSeed/core/Shader.h"
#include "TheSeed/ecs/Scene.h"
typedef struct {
  char *model_name;
  char *shader_name;
  TS_Shader *shader;
  unsigned int numMeshes;
  TS_Mesh **meshes;
} TS_Model;

void *ts_create_TS_Model();
void ts_destroy_TS_Model(void *ptr);
void ts_serialize_TS_Model(void *ptr, TS_Serialization *serialization);
void ts_deserialize_TS_Model(void *ptr, TS_Serialization *serialization);
void ts_activate_TS_Model(void *ptr);

#endif
