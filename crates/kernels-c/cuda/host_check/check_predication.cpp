// Host check: branchy vs predicated agent_update agree on random inputs (all states, vaccinated or not).
#include <cstdio>
#include <cstdint>
#include <cstdlib>
#include <cstring>
#include <cmath>
#include <algorithm>
#include <vector>
#define __global__
#define __device__
#define __forceinline__ inline
#define __shared__ static
struct uint4 { unsigned x,y,z,w; }; struct uint2 { unsigned x,y; };
inline uint4 make_uint4(unsigned a,unsigned b,unsigned c,unsigned d){return {a,b,c,d};}
struct dim3v { unsigned x,y,z; }; static dim3v threadIdx, blockIdx, blockDim, gridDim;
inline unsigned __umulhi(unsigned a, unsigned b){ return (unsigned)(((unsigned long long)a*b)>>32); }
inline unsigned long long __umul64hi(unsigned long long a, unsigned long long b){ return (unsigned long long)(((unsigned __int128)a*b)>>64); }
unsigned __shfl_up_sync(unsigned,unsigned,unsigned){return 0;} unsigned __shfl_down_sync(unsigned,unsigned,unsigned){return 0;}
unsigned __shfl_sync(unsigned,unsigned,unsigned){return 0;} unsigned __ballot_sync(unsigned,int){return 0;}
unsigned __clz(unsigned){return 0;} unsigned __ffs(unsigned){return 0;} unsigned __popc(unsigned){return 0;}
unsigned atomicAdd(unsigned*,unsigned){return 0;} unsigned atomicMin(unsigned*,unsigned){return 0;} unsigned atomicMax(unsigned*,unsigned){return 0;}
void __syncthreads(){}
using std::min;
#include "update.cu"
static uint64_t s=777;
static uint64_t rnd(){ s = s*6364136223846793005ULL+1442695040888963407ULL; return s>>11; }
int main(){
  P p; memset(&p,0,sizeof p);
  p.step=5; p.k0=11; p.k1=22; p.vstride=65; p.dcap=40;
  for(int a=0;a<5;a++){ p.thr_asym[a]=(uint64_t)(rnd()%4294967296ULL); p.thr_death[a]=(uint64_t)(rnd()%4294967296ULL);}  
  for(int k=0;k<4;k++){ p.don[k]=k==3?0:1; p.dcnt[k]=30; }
  std::vector<unsigned> dtab(4*40); for(int k=0;k<4;k++) for(int j=0;j<30;j++) dtab[k*40+j]=(unsigned)((j+1)*(4294967295.0/32));
  std::vector<uint64_t> hosp(5*65), binf(64);
  for(auto&h:hosp) h=rnd()%4294967296ULL; for(auto&b:binf){ double d=1.0-0.9*((double)rnd()/(double)(1ULL<<53)); memcpy(&b,&d,8);}  
  std::vector<uint64_t> thr(100), bits(100);
  for(int l=0;l<100;l++){ if(rnd()%3==0){thr[l]=bits[l]=0;} else { double lam=std::pow((double)rnd()/(double)(1ULL<<53),2.0)+1e-6; memcpy(&bits[l],&lam,8); thr[l]=(uint64_t)(lam*4294967296.0);} }
  GlobalLam lam={thr.data(),bits.data()};
  long bad=0,n=2000000;
  for(long k=0;k<n;k++){
    unsigned id=(unsigned)rnd(), age=rnd()%5, vs=rnd()%2, vd=rnd()%64, loc=rnd()%100;
    uint8_t st0=rnd()%7; uint16_t tm0 = (st0>=1&&st0<=4) ? (uint16_t)(rnd()%3) : 0;
    uint8_t s1=st0,s2=st0; uint16_t t1=tm0,t2=tm0; unsigned lp = (st0==6)?0xFFFFFFFFu:loc;
    bool n1=agent_update<false>(p,lam,dtab.data(),hosp.data(),binf.data(),id,age,&lp,s1,t1,vs,vd);
    bool n2=agent_update<true>(p,lam,dtab.data(),hosp.data(),binf.data(),id,age,&lp,s2,t2,vs,vd);
    if(n1!=n2||s1!=s2||t1!=t2){ if(bad<5) printf("diff st0=%d tm0=%d -> (%d,%d,%d) vs (%d,%d,%d)\n",st0,tm0,s1,t1,n1,s2,t2,n2); bad++; }
  }
  printf("branchy vs predicated mismatches: %ld / %ld\n",bad,n);
  return bad!=0;
}
