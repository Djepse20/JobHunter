#define Struct(Nam,...) typedef struct Nam Nam; struct Nam __VA_ARGS__

Struct(tomato,);

        typedef struct potato {
            struct potato * pPotato; 
            tomato* pTomato;

        }potato; 



typedef struct potato potato;

Struct(tomato, {
    potato* pPotato;
    tomato* pTomato;
});

int main() {
}