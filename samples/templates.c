#include <stdio.h>

#define _TEM_Array2D_Array3D_getA2D_getA3D_setA2D_setA3D(_T, _size) \
	typedef struct { \
		_T elements[_size][_size]; \
	} Array2D_##_T##_##_size; \
	typedef struct { \
		_T elements[_size][_size][_size]; \
	} Array3D_##_T##_##_size; \
	_T getA2D(Array2D_##_T##_##_size *a2d, int x, int y) { return a2d->elements[x][y]; } \
	_T getA3D(Array3D_##_T##_##_size *a3d, int x, int y, int z) { return a3d->elements[x][y][z]; } \
	void setA2D(Array2D_##_T##_##_size *a2d, int x, int y, _T it) { a2d->elements[x][y] = it; } \
	void setA3D(Array3D_##_T##_##_size *a3d, int x, int y, int z, _T it) { a3d->elements[x][y][z] = it; }

typedef struct { int id; } Tile;

#define _TEM_World_newWorld(_size) \
	_TEM_Array2D_Array3D_getA2D_getA3D_setA2D_setA3D(Tile, _size); \
	typedef struct { \
		int size; \
		Array2D_Tile_##_size grid; \
	} World_##_size; \
	World_##_size newWorld_##_size() { return (World_##_size){_size, (Array2D_Tile_##_size){}}; }

_TEM_World_newWorld(50);

int main() {
	const Tile tileAir = {0};
	const Tile tileStone = {1};
	const Tile tileGrass = {2};

	World_50 world = newWorld_50();
	Array2D_Tile_50 *pArray = &world.grid;

	// Fill the world
	for (int y = 0 ; y < world.size ; y++) {
		for (int x = 0 ; x < world.size ; x++) {
			if (y == 0 || x == 0 || y == world.size - 1 || x == world.size - 1) {
				setA2D(pArray, x, y, tileGrass);
			} else {
				setA2D(pArray, x, y, tileStone);
			}
		}
	}

	// Print the world
	for (int y = 0 ; y < world.size ; y++) {
		for (int x = 0 ; x < world.size ; x++) {
			printf("%d", getA2D(pArray, x, y).id);
		}
		printf("\n");
	}
}
