import type { Stats15Min } from '../stores/volatilityTypes'

export function getStitchedVolatilityProfile(
  allStats: Stats15Min[],
  targetHour: number,
  targetQuarter: number
): number[] {
  // Helper to find stats for a specific hour/quarter
  const findStats = (h: number, q: number) => 
    allStats.find(s => s.hour === h && s.quarter === q);

  // Calculate previous, current, next, next-next coordinates
  let prevQ = targetQuarter - 1;
  let prevH = targetHour;
  if (prevQ < 0) {
    prevQ = 3;
    prevH = (targetHour - 1 + 24) % 24;
  }

  let nextQ = targetQuarter + 1;
  let nextH = targetHour;
  if (nextQ > 3) {
    nextQ = 0;
    nextH = (targetHour + 1) % 24;
  }

  let nextNextQ = nextQ + 1;
  let nextNextH = nextH;
  if (nextNextQ > 3) {
    nextNextQ = 0;
    nextNextH = (nextH + 1) % 24;
  }

  const prevStats = findStats(prevH, prevQ);
  const currStats = findStats(targetHour, targetQuarter);
  const nextStats = findStats(nextH, nextQ);
  const nextNextStats = findStats(nextNextH, nextNextQ);

  const result: number[] = [];

  // 1. Previous Quarter: Last 5 minutes (indices 10-14)
  if (prevStats?.volatility_profile && prevStats.volatility_profile.length >= 15) {
    result.push(...prevStats.volatility_profile.slice(10, 15));
  } else {
    result.push(...Array(5).fill(0));
  }

  // 2. Current Quarter: All 15 minutes
  if (currStats?.volatility_profile && currStats.volatility_profile.length >= 15) {
    result.push(...currStats.volatility_profile);
  } else {
    result.push(...Array(15).fill(0));
  }

  // 3. Next Quarter: All 15 minutes
  if (nextStats?.volatility_profile && nextStats.volatility_profile.length >= 15) {
    result.push(...nextStats.volatility_profile);
  } else {
    result.push(...Array(15).fill(0));
  }

  // 4. Next Next Quarter: All 15 minutes (to reach +30 mins after end)
  if (nextNextStats?.volatility_profile && nextNextStats.volatility_profile.length >= 15) {
    result.push(...nextNextStats.volatility_profile);
  } else {
    result.push(...Array(15).fill(0));
  }

  return result;
}
