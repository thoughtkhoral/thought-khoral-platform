#!/usr/bin/env python3
# SPDX-License-Identifier: Apache-2.0
"""Deterministic external protocol executable; synthetic data only, no inference."""
import argparse, json, os, signal, subprocess, sys, time
parser = argparse.ArgumentParser()
parser.add_argument('--scenario', default='success')
parser.add_argument('--capture', required=True)
parser.add_argument('--version', action='store_true')
parser.add_argument('--persistent', action='store_true')
args, _ = parser.parse_known_args()
if args.version:
    print('codex-cli 0.160.0')
    sys.exit(0)

# Durable ownership registration precedes requests/holds. The outer owner can
# clean a group even if its leader died and only native descendants remain.
with open(args.capture + '.groups', 'a', encoding='utf-8') as groups:
    groups.write(str(os.getpgrp()) + '\n')

def emit(value):
    print(json.dumps(value, ensure_ascii=False), flush=True)

def response(request, value):
    emit({'id': request['id'], 'result': value})

def note(method, params):
    if method == 'item/completed':
        params = {**params, 'completedAtMs': 2}
    emit({'method': method, 'params': params})

def turn(status='inProgress', items=None):
    return {'id': 'turn-exact', 'items': items or [], 'status': status, 'error': None}

def model(id='model-a', name='gpt-6.1-sol'):
    return {'id': id, 'model': name, 'displayName': 'Synthetic model', 'description': 'Synthetic catalog', 'hidden': False, 'isDefault': id == 'model-a', 'defaultReasoningEffort': 'medium', 'supportedReasoningEfforts': [{'reasoningEffort': 'medium', 'description': 'Synthetic medium'}, {'reasoningEffort': 'high', 'description': 'Synthetic high'}]}
initialized = False
thread_id = 'thread-exact'
def hold(stage):
    if args.scenario == stage:
        subprocess.Popen([sys.executable, '-c', 'import time; time.sleep(60)', args.capture])
        with open(args.capture + '.' + stage + '.stage', 'w', encoding='utf-8') as marker:
            json.dump({'stage': stage, 'pid': os.getpid(), 'pgid': os.getpgrp()}, marker)
        while True:
            time.sleep(0.05)
for line in sys.stdin:
    request = json.loads(line)
    with open(args.capture, 'a', encoding='utf-8') as stream:
        stream.write(json.dumps({'request': request, 'environment': {'operatorSecretPresent': 'TASK4_OPERATOR_SECRET' in os.environ, 'hasProviderKey': 'OPENAI_API_KEY' in os.environ}, 'pid': os.getpid()}) + '\n')
    method = request['method']
    params = request.get('params', {})
    if method == 'initialize':
        assert not initialized
        response(request, {'userAgent': 'codex-cli/0.160.0', 'codexHome': os.environ.get('CODEX_HOME', '/synthetic'), 'platformFamily': 'unix', 'platformOs': 'linux'})
    elif method == 'initialized':
        initialized = True
    else:
        assert initialized
        if method == 'model/list':
            if params.get('cursor') is None:
                data = [model(), model('model-b', 'gpt-6-sol')]
                next_cursor = 'page-two'
                if args.scenario == 'duplicate_catalog':
                    data.append(model())
            else:
                assert params['cursor'] == 'page-two'
                data = [{**model('model-hidden', 'hidden-native'), 'hidden': True}, model('blocked', 'gpt-6-sol')]
                next_cursor = None
                if args.scenario == 'cycle_catalog':
                    next_cursor = 'page-two'
            response(request, {'data': data, 'nextCursor': next_cursor})
        elif method in ('thread/start', 'thread/resume'):
            hold('hold_thread_start')
            if args.scenario == 'runtime_unavailable':
                sys.exit(0)
            if args.persistent:
                from pathlib import Path
                storage = Path(os.environ['CODEX_HOME']) / 'sessions'
                storage.mkdir(exist_ok=True)
                if method == 'thread/start':
                    thread_id = 'thread-' + str(os.getpid())
                    (storage / (thread_id + '.jsonl')).write_text('synthetic native history')
                elif not (storage / (params['threadId'] + '.jsonl')).is_file():
                    emit({'id':request['id'],'error':{'code':-32000,'message':'synthetic missing native file'}})
                    continue
            if method == 'thread/resume':
                thread_id = params['threadId']
            thread = {'id': thread_id, 'cliVersion': '0.160.0', 'createdAt': 1, 'updatedAt': 1, 'cwd': params['cwd'], 'ephemeral': False, 'modelProvider': 'openai', 'preview': '', 'projectId': None, 'sessionId': 'synthetic-session', 'source': 'appServer', 'status': {'type': 'idle'}, 'turns': []}
            response(request, {'thread': thread, 'model': params['model'], 'reasoningEffort': None if args.scenario == 'null_effort' else params['config']['model_reasoning_effort'], 'modelProvider': 'openai', 'approvalPolicy': 'never', 'approvalsReviewer': 'user', 'cwd': params['cwd'], 'sandbox': {'type': 'readOnly', 'networkAccess': False}})
        elif method == 'turn/start':
            hold('hold_turn_start')
            if args.scenario == 'rpc_wrong_id':
                emit({'id': 999, 'result': {'turn': turn()}})
                continue
            if args.scenario == 'provider_denied':
                emit({'id':request['id'],'error':{'code':-32000,'message':'synthetic provider denial'}})
                continue
            response(request, {'turn': turn()})
            hold('hold_after_turn_start')
            if args.scenario in ('hang', 'ignore_interrupt', 'descendant'):
                if args.scenario == 'descendant':
                    child = subprocess.Popen([sys.executable, '-c', 'import time; time.sleep(60)'])
                    open(args.capture + '.child', 'w').write(str(child.pid))
                continue
            if args.scenario == 'early_exit':
                sys.exit(0)
            if args.scenario == 'malformed':
                print('{bad-json', flush=True)
                continue
            if args.scenario == 'duplicate_key':
                print('{"method":"turn/completed","method":"ignored","params":{}}', flush=True)
                continue
            if args.scenario == 'overflow':
                print('x' * 1048577, flush=True)
                continue
            if args.scenario == 'stderr_overflow':
                sys.stderr.write('x' * 4194305)
                sys.stderr.flush()
                continue
            if args.scenario == 'tool_request':
                emit({'id': 'tool-1', 'method': 'item/tool/call', 'params': {'threadId': thread_id, 'turnId': 'turn-exact'}})
                continue
            if args.scenario == 'approval':
                emit({'id': 'approval-1', 'method': 'item/commandExecution/requestApproval', 'params': {'threadId': thread_id, 'turnId': 'turn-exact'}})
                continue
            bound = 'wrong-thread' if args.scenario == 'wrong_thread' else thread_id
            notify = {'threadId': bound, 'turnId': 'wrong-turn' if args.scenario == 'wrong_turn' else 'turn-exact'}
            note('item/completed', {**notify, 'item': {'id': 'commentary-1', 'type': 'agentMessage', 'phase': 'commentary', 'text': 'Working privately'}})
            if args.scenario == 'delta_only':
                note('item/agentMessage/delta', {**notify, 'itemId': 'unfinished', 'delta': 'Not final'})
            elif args.scenario != 'commentary_only':
                message = {'id': 'final-1', 'type': 'agentMessage', 'text': 'Maya proposed green.'}
                if args.scenario != 'phase_missing':
                    message['phase'] = 'final_answer'
                if args.scenario == 'key_echo':
                    message['text'] = os.environ.get('OPENAI_API_KEY','missing-provider-key')
                if args.scenario == 'text_overflow':
                    message['text'] = 'é' * 32769
                note('item/completed', {**notify, 'item': message})
                if args.scenario == 'multiple_final':
                    note('item/completed', {**notify, 'item': {'id': 'final-2', 'type': 'agentMessage', 'phase': 'final_answer', 'text': 'Leo agreed.'}})
            if args.scenario in ('settings_switch', 'usage', 'reroute', 'compaction', 'wrong_reroute', 'unknown_reroute'):
                breakdown = {'totalTokens': 40, 'inputTokens': 30, 'outputTokens': 10, 'cachedInputTokens': 8, 'reasoningOutputTokens': 5}
                note('thread/tokenUsage/updated', {**notify, 'tokenUsage': {'last': breakdown, 'total': {**breakdown, 'totalTokens': 1000}, 'modelContextWindow': 100}})
                if args.scenario == 'settings_switch':
                    note('thread/settings/updated', {'threadId': thread_id, 'threadSettings': {'model': 'gpt-6-sol', 'modelProvider': 'openai', 'effort': 'medium', 'approvalPolicy': 'never', 'approvalsReviewer': 'user', 'cwd': params['cwd'], 'sandboxPolicy': {'type': 'readOnly', 'networkAccess': False}, 'collaborationMode': {'mode': 'default', 'settings': {'model': 'gpt-6-sol', 'reasoning_effort': 'medium', 'developer_instructions': None}}}})
                if args.scenario in ('reroute', 'wrong_reroute', 'unknown_reroute'):
                    note('model/rerouted', {**notify, 'turnId': 'wrong-turn' if args.scenario == 'wrong_reroute' else 'turn-exact', 'fromModel': 'gpt-6.1-sol', 'toModel': 'unknown-unreviewed-model' if args.scenario == 'unknown_reroute' else 'gpt-6.1-sol', 'reason': 'highRiskCyberActivity'})
                if args.scenario == 'compaction':
                    note('thread/compacted', notify)
            if args.scenario == 'noise':
                note('item/reasoning/textDelta', {**notify, 'itemId': 'reason-1', 'contentIndex': 0, 'delta': 'Private reasoning'})
                note('item/reasoning/summaryTextDelta', {**notify, 'itemId': 'reason-1', 'summaryIndex': 0, 'delta': 'Private summary'})
                note('thread/status/changed', {'threadId': thread_id, 'status': {'type': 'active', 'activeFlags': []}})
            if args.scenario == 'incomplete':
                continue
            status = 'failed' if args.scenario == 'failed' else 'interrupted' if args.scenario == 'interrupted' else 'completed'
            final = turn(status)
            if status == 'failed':
                final['error'] = {'message': 'Synthetic failure', 'additionalDetails': None, 'codexErrorInfo': None}
            if args.scenario == 'delayed_success':
                import time
                time.sleep(0.5)
            note('turn/completed', {'threadId': bound, 'turn': final})
        elif method == 'turn/interrupt':
            assert params == {'threadId': thread_id, 'turnId': 'turn-exact'}
            if args.scenario != 'ignore_interrupt':
                response(request, {})
                note('turn/completed', {'threadId': thread_id, 'turn': turn('interrupted')})
        else:
            raise AssertionError('Unsupported fake request ' + method)
