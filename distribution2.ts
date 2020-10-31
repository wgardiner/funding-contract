interface Vote {
  voter: string,
  amount: number,
  proposalId: string,
}

const votes: Vote[] = [
  {
    voter: 'voter_0',
    amount: 1,
    proposalId: 'proposal_0',
  },
  {
    voter: 'voter_1',
    amount: 4,
    proposalId: 'proposal_0',
  },
  {
    voter: 'voter_2',
    amount: 9,
    proposalId: 'proposal_1',
  },
  {
    voter: 'voter_0',
    amount: 16,
    proposalId: 'proposal_1',
  },
  {
    voter: 'voter_0',
    amount: 200,
    proposalId: 'proposal_2',
  },
  {
    voter: 'voter_1',
    amount: 1,
    proposalId: 'proposal_2',
  },
]

const proposals = [
  {
    id: 'proposal_0',
  },
  {
    id: 'proposal_1',
  },
  {
    id: 'proposal_2',
  },
];

const subsidyPoolConstrained = 50;



const reducedVotesByProp = votes.reduce((r, x) => {
  r[x.proposalId] = (r[x.proposalId] || 0) + x.amount;
  return r;
}, {});

const reducedVotes = votes.reduce((r, x) => {
  const tag = `${x.proposalId}--${x.voter}`;
  r[tag] = (r[tag] || 0) + x.amount;
  return r;
}, {});

// const reduceVotesByProp = (r: { [key: string]: Vote }, x: Vote) => {
//   const tag = `${x.proposalId}--${x.voter}`;
//   r[tag] = (r[tag] || 0) + x.amount;
//   return r;
// };

// const uniqueVotes = Object.values(votes.reduce((r, x) => {
//   const tag = `${x.proposalId}--${x.voter}`;
//   if (r[tag] === undefined) {
//     r[tag] = {
//       amount: 0,
//       voter: x.voter,
//       proposalId: x.proposalId,
//     };
//   }
//   r[tag].amount += x.amount;
//   return r;
// }, {}));


const reduceVotesByProperties = (properties: string[]) => (r: { [key: string]: Vote }, x: Vote) => {
  // const tag = `${x.proposalId}--${x.voter}`;
  const tag = properties.reduce((str, key) => str.concat(x[key], '--'), '');
  if (r[tag] === undefined) {
    r[tag] = {
      amount: 0,
      voter: x.voter,
      proposalId: x.proposalId,
    };
  }
  r[tag].amount += x.amount;
  return r;
}

const uniqueVotes = Object.values(
  votes.reduce(reduceVotesByProperties(['proposalId', 'voter']), {}),
);
// console.log('votes merged by vote-proposal pair', uniqueVotes);


const merged = proposals.map(p => {
  const votesForProp = uniqueVotes
    .filter(v => v.proposalId === p.id)
    .map(({ proposalId, ...rest }) => ({ ...rest }));

  return {
    ...p,
    votes: votesForProp,
  };
});

console.log('merged', JSON.stringify(merged, null, '  '));
// const collapseVotesByVoter = (votes: Vote[]) => votes.reduce((r, x) => {
//   r[x.id]
// }, {})
// const collapseVotesByVoter = (votes: Vote[]) => votes.reduce((r, x) => {
//   r = r.find(v => v.proposal)
// }, []);

const sum = (array) => array.reduce((r, x) => r + x, 0);
// calculate F^(p,LR)
// const calcIdealProjectDist = (projectContributions) => Math.pow(
//   sum(projectContributions.map(Math.sqrt)),
//   2
// );
const calcIdealProjectDist = (votes) => Math.pow(
  sum(votes.map(Math.sqrt)),
  2
);



// const calculateProjectDistActual = (projectContributions) =>
//   (calcIdealProjectDist(projectContributions) - sum(projectContributions))
//   / constraintFactor
//   + sum(projectContributions);

// const calculateProjectDistActual = (projectContributions) =>
//   (calcIdealProjectDist(projectContributions) - sum(projectContributions))
//   / (sum(allSubsidiesIdealAlt) / subsidyPoolConstrained)
//   + sum(projectContributions);

// const calculateProjectDistActual = (projectContributions) =>
//   (calcIdealProjectDist(projectContributions) - sum(projectContributions))
//   / (sum(allSubsidiesIdealAlt) / subsidyPoolConstrained)
//   + sum(projectContributions);

const simplified = merged
  .map(p => ({
    id: p.id,
    votes: p.votes.map(v => v.amount)
  }));


// const constraintFactor = Math.pow(sum(simplified.map(Math.sqrt)), 2)


const idealResults = simplified.map((p) => {
  const distributionIdeal = calcIdealProjectDist(p.votes);
  const subsidyIdeal = distributionIdeal - sum(p.votes);
  // const constraintFactor = arr.reduce((r, x) => )
  // // const distributionActual = (distributionIdeal - sum(p.votes)) /
  return {
    ...p,
    distributionIdeal,
    subsidyIdeal,
  };
})

const constraintFactor = sum(idealResults.map((p) => p.subsidyIdeal)) / subsidyPoolConstrained;
console.log('constraint factor', constraintFactor);

const results = idealResults.map((p) => {
  const totalVotes = sum(p.votes);
  const distributionActual = (p.distributionIdeal - totalVotes) / constraintFactor + totalVotes;
  return {
    ...p,
    distributionActual,
    totalVotes,
    subsidyActual: distributionActual - totalVotes,
  }
})

console.log('results', JSON.stringify(results, null, '  '));
