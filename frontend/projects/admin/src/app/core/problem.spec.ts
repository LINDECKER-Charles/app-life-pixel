import { ApiProblem } from 'shared';
import { asProblem, emptyReason, problemMessage } from './problem';

describe('problem', () => {
  it('says why a view is empty', () => {
    expect(emptyReason(null)).toBe('nothing');
    expect(emptyReason(new ApiProblem(503, 'monitoring.not_configured'))).toBe('not_configured');
    expect(emptyReason(new ApiProblem(503, 'monitoring.unavailable'))).toBe('unavailable');
    expect(emptyReason(new ApiProblem(0, 'service.unavailable'))).toBe('unavailable');
    expect(emptyReason(new ApiProblem(500, 'internal.error'))).toBe('failed');
  });

  it('translates a problem by its code', () => {
    const problem = new ApiProblem(503, 'monitoring.unavailable', { source: 'loki' });

    expect(problemMessage(problem)).toEqual({
      key: 'errors.monitoring.unavailable',
      params: { source: 'loki' },
    });
  });

  it('keeps a problem, and turns anything else into an internal error', () => {
    const problem = new ApiProblem(400, 'request.malformed');

    expect(asProblem(problem)).toBe(problem);
    expect(asProblem(new TypeError('boom')).code).toBe('internal.error');
  });
});
