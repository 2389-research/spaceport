"""Specification-tool tests only. These do not run Spaceport or Herdr."""
from __future__ import annotations
import copy
import json
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path
import spec


class SpecificationToolsTests(unittest.TestCase):
    def test_duplicate_keys_are_rejected(self):
        with self.assertRaises(ValueError):
            json.loads('{"x":1,"x":2}',object_pairs_hook=spec.unique_object)

    def test_nonfinite_json_is_rejected(self):
        with self.assertRaises(ValueError):
            json.loads('{"x":NaN}',parse_constant=spec.reject_constant)

    def test_duplicate_ids_are_rejected(self):
        with self.assertRaises(ValueError):
            spec.index([{'id':'A'},{'id':'A'}],'fixture')

    def test_cycles_are_rejected(self):
        with self.assertRaises(ValueError):
            spec.validate_graph({'A':{'depends_on':['B']},'B':{'depends_on':['A']}})

    def test_missing_prerequisites_are_rejected(self):
        with self.assertRaises(ValueError):
            spec.validate_graph({'A':{'depends_on':['missing']}})

    def test_acyclic_graph_is_accepted(self):
        spec.validate_graph({'A':{'depends_on':[]},'B':{'depends_on':['A']}})

    def test_packet_rejects_unknown_task(self):
        with self.assertRaises(ValueError):
            spec.task_packet(spec.ROOT,'DOES-NOT-EXIST')

    def test_packet_refuses_silent_truncation(self):
        with self.assertRaises(ValueError):
            spec.task_packet(spec.ROOT,'T00',100)

    def test_packet_contains_required_context(self):
        packet=spec.task_packet(spec.ROOT,'T00')
        self.assertIn('SP-025',packet)
        self.assertIn('input_sha256',packet)
        self.assertIn('No broad code deletion',packet)
        self.assertIn('"status": "passed"',packet)
        self.assertNotIn('All application scenarios begin not_run',packet)

    def test_all_packets_fit_default_budget(self):
        for task in spec.registries(spec.ROOT)['tasks']:
            self.assertLessEqual(len(spec.task_packet(spec.ROOT,task).encode()),65536)

    def test_input_change_changes_hash(self):
        with tempfile.TemporaryDirectory() as d:
            root=Path(d)
            (root/'a.md').write_text('first')
            before=spec.input_hashes(root)
            (root/'a.md').write_text('second')
            self.assertNotEqual(before,spec.input_hashes(root))

    def test_read_rejects_path_escape(self):
        with self.assertRaises(ValueError):
            spec.text(spec.ROOT,'../build_spaceport_spec.py')

    def test_evidence_must_name_an_existing_file(self):
        with tempfile.TemporaryDirectory() as raw:
            root=Path(raw)
            (root/'evidence').mkdir()
            (root/'evidence/run.md').write_text('ok',encoding='utf-8')
            spec.validate_evidence(root,{'id':'A','evidence':['evidence/run.md']})
            with self.assertRaises(ValueError) as missing:
                spec.validate_evidence(root,{'id':'A','evidence':['evidence/gone.md']})
            self.assertIn('A',str(missing.exception))
            self.assertIn('evidence/gone.md',str(missing.exception))
            with self.assertRaises(ValueError):
                spec.validate_evidence(root,{'id':'A','evidence':['../outside.md']})

    def test_unknown_outcome_cannot_be_retry_safe(self):
        from jsonschema import Draft202012Validator
        schema=spec.load(spec.ROOT/'contracts/control.schema.json')
        example=spec.load(spec.ROOT/'examples/unknown-outcome.json')
        bad=copy.deepcopy(example); bad['retry_safe']=True
        self.assertFalse(Draft202012Validator(schema).is_valid(bad))

    def test_observed_outcome_needs_evidence(self):
        from jsonschema import Draft202012Validator
        schema=spec.load(spec.ROOT/'contracts/control.schema.json')
        example=spec.load(spec.ROOT/'examples/unknown-outcome.json')
        bad=copy.deepcopy(example); bad['state']='observed';bad['evidence']=[]
        self.assertFalse(Draft202012Validator(schema).is_valid(bad))

    def test_investigation_can_complete_with_recorded_open_gap(self):
        task={'id':'T','status':'done','evidence':['documented findings'],
              'gaps':['G'],'must_close_gaps':[]}
        spec.validate_task_completion(task,{'G':{'status':'open'}})

    def test_parity_task_cannot_complete_with_open_gap(self):
        task={'id':'T','status':'done','evidence':['source notes'],
              'gaps':['G'],'must_close_gaps':['G']}
        with self.assertRaises(ValueError):
            spec.validate_task_completion(task,{'G':{'status':'open'}})

    def test_done_task_requires_evidence(self):
        task={'id':'T','status':'done','evidence':[],
              'gaps':[],'must_close_gaps':[]}
        with self.assertRaises(ValueError):
            spec.validate_task_completion(task,{})

    def test_skipped_schema_is_reported_not_run(self):
        report=spec.validate(spec.ROOT,require_schema=False)
        self.assertEqual(report['schema_validation'],'not_run')
        self.assertEqual(report['schema_examples'],0)

    def test_environment_is_not_a_specification_input(self):
        with tempfile.TemporaryDirectory() as d:
            root=Path(d); (root/'.venv').mkdir()
            (root/'.venv'/'installed.json').write_text('not specification JSON')
            (root/'SPEC.md').write_text('fixture')
            self.assertEqual(list(spec.input_hashes(root)), ['SPEC.md'])

    def test_hashing_rejects_symlink_inputs(self):
        with tempfile.TemporaryDirectory() as d:
            root=Path(d); (root/'alias').symlink_to('/etc/passwd')
            with self.assertRaises(ValueError):
                spec.input_hashes(root)

    def test_full_package(self):
        report=spec.validate(spec.ROOT)
        self.assertEqual(report['result'],'passed')
        scenarios=spec.load(spec.ROOT/'plan/acceptance.json')['scenarios']
        executed=sum(item['status'] in {'passed','failed'} for item in scenarios)
        self.assertEqual(report['application_scenarios_executed'],executed)

    def test_runnable_task_needs_started_prerequisites(self):
        tasks={'A':{'id':'A','status':'done','depends_on':[]},
               'B':{'id':'B','status':'in_progress','depends_on':['A']},
               'C':{'id':'C','status':'not_started','depends_on':['B']},
               'D':{'id':'D','status':'blocked','depends_on':['A']},
               'E':{'id':'E','status':'not_started','depends_on':['D']}}
        self.assertEqual(spec.runnable_tasks(tasks),{'A','B','C'})

    def test_shared_scenario_owners_keep_plan_order(self):
        tasks={'T01':{'id':'T01','acceptance':['A01','A02']},'T02':{'id':'T02','acceptance':['A02']}}
        self.assertEqual(spec.scenario_owners(tasks),{'A01':['T01'],'A02':['T01','T02']})

    def test_broken_evidence_names_missing_and_escaping_paths(self):
        with tempfile.TemporaryDirectory() as raw:
            root=Path(raw)
            (root/'evidence').mkdir()
            (root/'evidence/run.md').write_text('ok',encoding='utf-8')
            self.assertEqual(spec.broken_evidence(root,{'id':'A','evidence':['evidence/run.md']}),[])
            self.assertEqual(spec.broken_evidence(root,{'id':'A','evidence':['evidence/gone.md','../outside.md']}),
                             ['evidence/gone.md','../outside.md'])

    def test_scenarios_view_covers_every_scenario_once(self):
        view=spec.scenario_view(spec.ROOT)
        acceptance=spec.registries(spec.ROOT)['acceptance']
        listed=[s['id'] for group in view['groups'] for s in group['scenarios']]
        listed+=[s['id'] for s in view['unassigned']]
        self.assertEqual(sorted(listed),sorted(acceptance))
        self.assertEqual(sum(view['counts'].values()),view['total'])
        self.assertEqual(view['total'],len(acceptance))
        self.assertEqual(view['counts']['not_run']+view['counts']['passed']
                         +view['counts']['failed']+view['counts']['blocked'],len(acceptance))

    def scenario_report_view(self,runnable=True):
        task={'id':'T01','title':'Task one','status':'in_progress'}
        pending={'id':'A01','title':'One','status':'not_run','requirements':['SP-001']}
        passed={'id':'A02','title':'Two','status':'passed','requirements':['SP-002']}
        return {'counts':{'not_run':1,'passed':1,'failed':0,'blocked':0},'total':2,
                'broken':{'A02':['evidence/gone.md']},'owners':{'A01':['T01'],'A02':['T01','T02']},
                'runnable':1 if runnable else 0,
                'groups':[{'task':task,'scenarios':[pending,passed],'runnable':runnable}],
                'unassigned':[]}

    def test_scenarios_report_groups_by_task_with_requirements(self):
        report=spec.scenario_report(self.scenario_report_view())
        self.assertIn('not_run 1',report)
        self.assertIn('passed 1',report)
        self.assertIn('T01 — Task one',report)
        self.assertIn('SP-001',report)
        self.assertIn('also T02',report)

    def test_scenarios_report_flags_broken_evidence(self):
        report=spec.scenario_report(self.scenario_report_view())
        self.assertIn('Evidence references that are not files here: 1',report)
        self.assertIn('A02 (T01, T02): evidence/gone.md',report)
        self.assertIn('[evidence not a file: evidence/gone.md]',report)

    def test_scenarios_report_status_filter_hides_other_statuses(self):
        report=spec.scenario_report(self.scenario_report_view(),{'passed'})
        self.assertIn('A02',report)
        self.assertNotIn('A01  not_run',report)
        self.assertIn('No scenario matches the requested filter.',
                      spec.scenario_report(self.scenario_report_view(),set()))

    def test_scenarios_next_keeps_only_runnable_not_run(self):
        report=spec.scenario_report(self.scenario_report_view(),next_only=True)
        self.assertIn('Next runnable scenario: A01 — One (T01)',report)
        self.assertIn('A01  not_run',report)
        self.assertNotIn('A02  passed',report)
        blocked=spec.scenario_report(self.scenario_report_view(False),next_only=True)
        self.assertIn('No runnable not_run scenario remains.',blocked)
        self.assertNotIn('A01  not_run',blocked)
        self.assertIn('No scenario matches the requested filter.',
                      spec.scenario_report(self.scenario_report_view(),{'passed'},next_only=True))

    def test_scenario_status_parsing(self):
        self.assertIsNone(spec.parse_statuses(None))
        self.assertEqual(spec.parse_statuses(['passed,blocked']),{'passed','blocked'})
        self.assertEqual(spec.parse_statuses(['passed','failed']),{'passed','failed'})
        with self.assertRaises(ValueError):
            spec.parse_statuses(['nope'])
        with self.assertRaises(ValueError):
            spec.parse_statuses([','])

    def test_scenarios_strict_exits_nonzero_only_while_not_run(self):
        pending=spec.registries(spec.ROOT)['acceptance']
        remaining=sum(s['status']=='not_run' for s in pending.values())
        result=subprocess.run([sys.executable,str(spec.ROOT/'tools/spec.py'),'scenarios','--strict'],
                              capture_output=True,text=True,check=False)
        self.assertEqual(result.returncode,1 if remaining else 0)
        if remaining:
            self.assertIn(f'Acceptance gate: {remaining} scenarios are not_run',result.stderr)
        plain=subprocess.run([sys.executable,str(spec.ROOT/'tools/spec.py'),'scenarios'],
                             capture_output=True,text=True,check=False)
        self.assertEqual(plain.returncode,0)


if __name__=='__main__':
    unittest.main(verbosity=2)
