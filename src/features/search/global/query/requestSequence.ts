/** Numbers requests in the order they were sent, so answers to outdated requests can be dropped. */
export type RequestSequence = {
  /** Starts a new request and returns its ticket; every earlier ticket becomes outdated. */
  begin: () => number;
  /** True while no newer request was started after the one with this ticket. */
  isLatest: (ticket: number) => boolean;
};

export function createRequestSequence(): RequestSequence {
  let latestTicket = 0;

  return {
    begin: () => {
      latestTicket += 1;
      return latestTicket;
    },
    isLatest: (ticket) => ticket === latestTicket,
  };
}
