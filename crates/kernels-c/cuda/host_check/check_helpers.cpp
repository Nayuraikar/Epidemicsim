// Host-side check of the device helpers in common.cuh (intrinsics emulated), no GPU involved.
#include <cstdio>
#include <cstdint>
#include <cstdlib>
#include <cstring>
#include <cmath>
#define __global__
#define __device__
#define __forceinline__ inline
#define __shared__ static
#define alignas_dummy
struct uint4 { unsigned x,y,z,w; }; struct uint2 { unsigned x,y; };
inline uint4 make_uint4(unsigned a,unsigned b,unsigned c,unsigned d){return {a,b,c,d};}
inline unsigned __umulhi(unsigned a, unsigned b){ return (unsigned)(((unsigned long long)a*b)>>32); }
inline unsigned long long __umul64hi(unsigned long long a, unsigned long long b){ return (unsigned long long)(((unsigned __int128)a*b)>>64); }
#include "common.cuh"
static uint64_t s=12345;
static uint64_t rnd(){ s = s*6364136223846793005ULL+1442695040888963407ULL; return s>>11; }
int main(){
  uint4 r = philox(0x243f6a88u,0x85a308d3u,0x13198a2eu,0x03707344u,0xa4093822u,0x299f31d0u);
  printf("philox KAT: %08x %08x %08x %08x\n", r.x,r.y,r.z,r.w);
  long bad=0, n=3000000;
  for (long k=0;k<n;k++){
    double lam = std::pow((double)rnd()/(double)(1ULL<<53),3.0); if(lam<1e-9) lam=1e-9;
    double ve = ((double)rnd()/(double)(1ULL<<53))*0.95, bb = 1.0-ve;
    double prod = lam*bb;
    // boundary-adjacent x half of the time
    unsigned x;
    if (k%2==0) x=(unsigned)rnd(); else { double c=std::floor(prod*4294967296.0); long long ci=(long long)c + (long long)(rnd()%5) - 2; if(ci<0)ci=0; if(ci>4294967295LL)ci=4294967295LL; x=(unsigned)ci; }
    uint64_t a,b; memcpy(&a,&lam,8); memcpy(&b,&bb,8);
    bool want = ((double)x+0.5)*(1.0/4294967296.0) < prod;
    if (want != lt_prod(x,a,b)) bad++;
  }
  printf("lt_prod mismatches: %ld / %ld\n", bad, n);
  return 0;
}
