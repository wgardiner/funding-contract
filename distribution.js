// funding

// array of contributions,
// C := [C^p0, C^p1, C^p2, ... ] = [[c^p0_0, c^p0_1, ...], [c^p1_0, c^p1_1, ...], ...]
const allContributions = [
	[
		1,
		4,
	],
	[
		9,
		16,
	]
];

// D
const subsidyPoolConstrained = 50;

const sum = (array) => array.reduce((r, x) => r + x, 0);

// calculate F^(p,LR)
const calcIdealProjectDist = (projectContributions) => Math.pow(
    sum(projectContributions.map(Math.sqrt)),
    2
  );

// F^(P,LR)
const allDistributionsIdeal = allContributions.map(calcIdealProjectDist);
console.log('all dist ideal', allDistributionsIdeal);


// M^(P,LR)
// Array of M^(p, LR)= F^(p,LR) - C^p
// where C^p = sum(c^p)
const allSubsidiesIdeal = allContributions.map(
  (projectContributions) =>
    calcIdealProjectDist(projectContributions) - sum(projectContributions)
);
// Alternative implementation M^(p,LR) = Σ_i(√(c^p_i))**2 - Σ_i(c^i_p)
const calculateProjectSubsidyIdealAlt = (projectContributions) =>
  Math.pow(sum(projectContributions.map(Math.sqrt)), 2) - sum(projectContributions);
const allSubsidiesIdealAlt = allContributions.map(calculateProjectSubsidyIdealAlt);

console.log('all subsidies ideal', allSubsidiesIdeal);
console.log('all subsidies ideal alt', allSubsidiesIdealAlt);

// D = (1 / k) * sum_p(M^(p,LR))
// k = sum_p(M^(p,LR)) / D
const constraintFactor = sum(allSubsidiesIdealAlt) / subsidyPoolConstrained;
console.log('constraint factor', constraintFactor);

const calculateProjectDistActual = (projectContributions) =>
  // const projectDistIdeal = calcIdealProjectDist(projectContributions);
  (calcIdealProjectDist(projectContributions) - sum(projectContributions))
  / constraintFactor
  + sum(projectContributions);

const allDistributionsActual = allContributions.map(calculateProjectDistActual);

console.log('all dist actual', allDistributionsActual);
console.log('all match actual', allDistributionsActual.map((x, i) => x - sum(allContributions[i])));
console.log('test', sum(allDistributionsActual) - sum(allContributions.map(sum)));
