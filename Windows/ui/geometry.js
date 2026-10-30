// 只包含绘制和手势逻辑；额度、计时和持久状态由宿主管理。
export const digits = value => Number.isFinite(value) ? String(Math.round(Math.max(0, Math.min(100, value)))) : '—';
export const name = minutes => minutes === 300 ? '每 5 小时' : minutes === 10080 ? '每周' : minutes ? `${minutes} 分钟` : '当前窗口';
export const shortName = minutes => minutes === 300 ? '5h' : minutes === 10080 ? '周' : '额度';
export const color = value => value === null || value === undefined ? '#89929a' : value <= 10 ? '#e25152' : value <= 25 ? '#dd9d2a' : '#40aa83';
export function halfPath(fraction, lower = false) {
  const r=15.5, straight=16, arc=Math.PI*r;
  let distance=Math.max(0,Math.min(1,fraction))*(2*straight+arc);
  const points=[[2.5,34]];
  const add=(x,y)=>points.push([x,y]);
  add(2.5,34-Math.min(straight,distance)); distance-=straight;
  if(distance>0) {
    const angle=Math.min(arc,distance)/r;
    for(let step=1;step<=Math.ceil(angle*20);step++) {
      const theta=Math.min(angle,step/20); add(18-r*Math.cos(theta),18-r*Math.sin(theta));
    }
    distance-=arc;if(distance>0)add(33.5,18+Math.min(straight,distance));
  }
  return points.map(([x,y],i)=>`${i?'L':'M'}${lower?36-x:x},${lower?72-y:y}`).join(' ');
}
export class Gesture {
  constructor(threshold=4){this.threshold=threshold;this.cancel();}
  down(x,y){this.origin=[x,y];this.dragged=false;}
  move(x,y){if(!this.origin||this.dragged)return false;if(Math.abs(x-this.origin[0])>=this.threshold||Math.abs(y-this.origin[1])>=this.threshold){this.dragged=true;return true;}return false;}
  up(){const click=!!this.origin&&!this.dragged;this.cancel();return click;}
  cancel(){this.origin=null;this.dragged=false;}
}
